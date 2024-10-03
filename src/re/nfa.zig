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
begin: usize,
final: std.ArrayList(bool),

pub fn init(allocator: std.mem.Allocator, charset: Charset, pattern: []const u8) !NFA {
    var nfa = NFA{
        .allocator = allocator,
        .nodes = std.ArrayList(std.AutoHashMap(usize, Charset)).init(allocator),
        .begin = 0,
        .final = std.ArrayList(bool).init(allocator),
    };
    var lex = Lexer{
        .pattern = pattern,
        .charset = charset,
        .i = 0,
    };
    const s = nfa.compile(&lex, 0) catch |e| {
        nfa.deinit();
        return e;
    };
    nfa.final.appendNTimes(false, nfa.nodes.items.len) catch |e| {
        nfa.deinit();
        return e;
    };
    nfa.begin = s.begin;
    nfa.final.items[s.end] = true;
    nfa.removeEpsilons() catch |e| {
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
    nfa.final.deinit();
}

pub fn graph(nfa: NFA) !Graph {
    var g = try Graph.init(nfa.allocator, nfa.nodes.items.len);
    for (nfa.nodes.items, 0..) |n, i| {
        g.node(i, i == nfa.begin, nfa.final.items[i]);
        var it = n.iterator();
        while (it.next()) |e| {
            g.edge(i, e.key_ptr.*, e.value_ptr.*);
        }
    }
    return g;
}

// Build 1-NFA

fn removeEpsilons(nfa: *NFA) !void {
    const used = try nfa.allocator.alloc(bool, nfa.nodes.items.len);
    defer nfa.allocator.free(used);
    for (0..nfa.nodes.items.len) |i| {
        @memset(used, false);
        try nfa.epsilonDfs(used, i, i);
    }
    for (nfa.nodes.items) |n| {
        var it = n.iterator();
        while (it.next()) |e| {
            e.value_ptr.ipop(0);
        }
    }
    @memset(used, false);
    nfa.visitDfs(used, nfa.begin);
    const nodes = nfa.nodes.items.len;
    for (0..nodes) |n| {
        if (!used[nodes - n - 1]) {
            try nfa.removeNode(nodes - n - 1);
            const g = try nfa.graph();
            defer g.deinit();
            try g.flush(std.io.getStdOut().writer().any());
        }
    }
}

fn epsilonDfs(nfa: *NFA, used: []bool, p: usize, n: usize) !void {
    used[n] = true;
    var it = nfa.nodes.items[n].iterator();
    while (it.next()) |e| {
        const i = e.key_ptr.*;
        if (!e.value_ptr.contains(0) or used[i]) {
            continue;
        }
        try epsilonDfs(nfa, used, p, i);
        if (nfa.final.items[i]) {
            nfa.final.items[n] = true;
        }
    }
    it = nfa.nodes.items[n].iterator();
    while (it.next()) |e| {
        (try nfa.getEdge(p, e.key_ptr.*)).imerge(e.value_ptr.*);
    }
}

fn visitDfs(nfa: *NFA, used: []bool, n: usize) void {
    used[n] = true;
    var it = nfa.nodes.items[n].iterator();
    while (it.next()) |e| {
        if (!used[e.key_ptr.*] and !e.value_ptr.empty()) {
            nfa.visitDfs(used, e.key_ptr.*);
        }
    }
}

fn removeNode(nfa: *NFA, n: usize) !void {
    const i = nfa.nodes.items.len - 1;
    if (n != i) {
        nfa.final.items[n] = nfa.final.items[i];
        for (0..nfa.nodes.items.len) |p| {
            if (p != n) {
                if (nfa.nodes.items[p].get(i)) |s| {
                    try nfa.nodes.items[p].put(n, s);
                }
            }
        }
        const p = nfa.nodes.items[i];
        nfa.nodes.items[i] = nfa.nodes.items[n];
        nfa.nodes.items[n] = p;
    }
    for (nfa.nodes.items) |*p| {
        _ = p.remove(i);
    }
    var v = nfa.nodes.pop();
    v.deinit();
    _ = nfa.final.pop();
}

// Compile NFA

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
