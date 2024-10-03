const std = @import("std");
const Charset = @import("charset.zig");
const Lexer = @import("lex.zig");
const Graph = @import("graph.zig");

const NFA = @This();

const ParseError = error{
    Balance,
    UnknownToken,
    BadExpr,
};

const Slice = struct {
    begin: usize,
    end: usize,
};

allocator: std.mem.Allocator,
nodes: std.ArrayList(std.AutoHashMap(usize, Charset)),
slice: Slice,

pub fn init(allocator: std.mem.Allocator, charset: Charset, pattern: []const u8) !NFA {
    var nfa = NFA{
        .allocator = allocator,
        .nodes = std.ArrayList(std.AutoHashMap(usize, Charset)).init(allocator),
        .slice = undefined,
    };
    var lex = Lexer{
        .pattern = pattern,
        .charset = charset,
        .i = 0,
    };
    nfa.slice = nfa.compile(&lex, 0) catch |e| {
        nfa.deinit();
        return e;
    };
    return nfa;
}

pub fn deinit(nfa: NFA) void {
    for (nfa.nodes.items) |*n| {
        n.deinit();
    }
    nfa.nodes.deinit();
}

pub fn graph(nfa: NFA) !Graph {
    var g = try Graph.init(nfa.allocator, nfa.nodes.items.len);
    for (nfa.nodes.items, 0..) |n, i| {
        g.node(i, i == nfa.slice.begin, i == nfa.slice.end);
        var it = n.iterator();
        while (it.next()) |e| {
            g.edge(i, e.key_ptr.*, e.value_ptr.*);
        }
    }
    return g;
}

fn compile(nfa: *NFA, lexer: *Lexer, level: u32) !Slice {
    var queue = std.ArrayList(Slice).init(nfa.allocator);
    defer queue.deinit();
    var concats: usize = 0;
    var cbrace: bool = false;
    while (try lexer.token()) |t| {
        switch (t.type) {
            '*' => try queue.append(try nfa.star(queue.pop())),
            '+' => (try nfa.getEdge(queue.getLast().end, queue.getLast().begin)).iadd(0),
            '?' => (try nfa.getEdge(queue.getLast().begin, queue.getLast().end)).iadd(0),
            '|' => {
                for (1..concats) |_| {
                    const b = queue.pop();
                    try queue.append(try nfa.concatenate(queue.pop(), b));
                }
                concats = 0;
            },
            '.' => {
                concats += 1;
                try queue.append(try nfa.sliceFromCharset(t.charset));
            },
            '(' => {
                concats += 1;
                try queue.append(try nfa.compile(lexer, level + 1));
            },
            ')' => {
                cbrace = true;
                break;
            },
            else => return ParseError.UnknownToken,
        }
    }
    if ((level == 0 and cbrace) or (level != 0 and !cbrace)) {
        return ParseError.Balance;
    }
    for (1..concats) |_| {
        const b = queue.pop();
        try queue.append(try nfa.concatenate(queue.pop(), b));
    }
    if (queue.items.len == 0) {
        return ParseError.BadExpr;
    }
    while (queue.items.len > 1) {
        const b = queue.pop();
        try queue.append(try nfa.merge(queue.pop(), b));
    }
    return queue.items[0];
}

fn sliceFromCharset(nfa: *NFA, charset: Charset) !Slice {
    const begin = try nfa.node();
    const end = try nfa.node();
    try nfa.nodes.items[begin].put(end, charset);
    return .{ .begin = begin, .end = end };
}

fn node(nfa: *NFA) !usize {
    try nfa.nodes.append(std.AutoHashMap(usize, Charset).init(nfa.allocator));
    return nfa.nodes.items.len - 1;
}

fn getEdge(nfa: *NFA, a: usize, b: usize) !*Charset {
    return (try nfa.nodes.items[a].getOrPut(b)).value_ptr;
}

fn concatenate(nfa: *NFA, a: Slice, b: Slice) !Slice {
    (try nfa.getEdge(a.end, b.begin)).iadd(0);
    return .{ .begin = a.begin, .end = b.end };
}

fn merge(nfa: *NFA, a: Slice, b: Slice) !Slice {
    const n1 = try nfa.node();
    (try nfa.getEdge(n1, a.begin)).iadd(0);
    (try nfa.getEdge(n1, b.begin)).iadd(0);
    const n2 = try nfa.node();
    (try nfa.getEdge(a.end, n2)).iadd(0);
    (try nfa.getEdge(b.end, n2)).iadd(0);
    return .{ .begin = n1, .end = n2 };
}

fn star(nfa: *NFA, a: Slice) !Slice {
    const n = try nfa.node();
    (try nfa.getEdge(n, a.begin)).iadd(0);
    (try nfa.getEdge(a.end, n)).iadd(0);
    return .{ .begin = n, .end = n };
}
