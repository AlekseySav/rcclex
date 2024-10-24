const std = @import("std");
const Charset = @import("charset.zig");
const Lexer = @import("lexer.zig");

const Self = @This();

const ParserError = error{
    BadBraceBalance,
    BadExpr,
};

const Slice = struct {
    begin: usize,
    final: usize,
};

const Edge = struct {
    a: usize,
    b: usize,
    c: Charset,
};

alloc: std.mem.Allocator,
edges: std.ArrayList(Edge),
slice: Slice,
nodes: usize,
epsilon: u8,

pub fn init(a: std.mem.Allocator, eps: u8) Self {
    return .{
        .alloc = a,
        .edges = std.ArrayList(Edge).init(a),
        .slice = undefined,
        .nodes = 0,
        .epsilon = eps,
    };
}

pub fn build(s: *Self, lex: *Lexer) !void {
    s.slice = try s.compile(lex, 0);
}

pub fn deinit(nfa: Self) void {
    nfa.edges.deinit();
}

fn compile(nfa: *Self, lexer: *Lexer, level: u32) !Slice {
    var queue = std.ArrayList(Slice).init(nfa.alloc);
    defer queue.deinit();
    var concats: usize = 0;
    var cbrace: bool = false;
    while (try lexer.token()) |t| {
        switch (t.type) {
            '*' => try queue.append(try nfa.star(queue.pop())),
            '+' => try nfa.edge(queue.getLast().final, queue.getLast().begin, Charset.char(nfa.epsilon)),
            '?' => try nfa.edge(queue.getLast().begin, queue.getLast().final, Charset.char(nfa.epsilon)),
            '|' => {
                try nfa.applyConcats(concats, &queue);
                concats = 0;
            },
            '.' => {
                concats += 1;
                try queue.append(try nfa.charset(t.charset));
            },
            '(' => {
                concats += 1;
                try queue.append(try nfa.compile(lexer, level + 1));
            },
            ')' => {
                cbrace = true;
                break;
            },
            else => unreachable,
        }
    }
    if ((level == 0 and cbrace) or (level != 0 and !cbrace)) {
        return ParserError.BadBraceBalance;
    }
    try nfa.applyConcats(concats, &queue);
    if (queue.items.len == 0) {
        return ParserError.BadExpr;
    }
    while (queue.items.len > 1) {
        const b = queue.pop();
        try queue.append(try nfa.merge(queue.pop(), b));
    }
    return queue.items[0];
}

fn edge(nfa: *Self, a: usize, b: usize, s: Charset) !void {
    try nfa.edges.append(.{ .a = a, .b = b, .c = s });
}

fn node(nfa: *Self) usize {
    nfa.nodes += 1;
    return nfa.nodes - 1;
}

fn charset(nfa: *Self, c: Charset) !Slice {
    const s = Slice{ .begin = nfa.node(), .final = nfa.node() };
    try nfa.edge(s.begin, s.final, c);
    return s;
}

fn concat(nfa: *Self, a: Slice, b: Slice) !Slice {
    try nfa.edge(a.final, b.begin, Charset.char(nfa.epsilon));
    return .{ .begin = a.begin, .final = b.final };
}

fn merge(nfa: *Self, a: Slice, b: Slice) !Slice {
    const s = Slice{ .begin = nfa.node(), .final = nfa.node() };
    try nfa.edge(s.begin, a.begin, Charset.char(nfa.epsilon));
    try nfa.edge(s.begin, b.begin, Charset.char(nfa.epsilon));
    try nfa.edge(a.final, s.final, Charset.char(nfa.epsilon));
    try nfa.edge(b.final, s.final, Charset.char(nfa.epsilon));
    return s;
}

fn star(nfa: *Self, a: Slice) !Slice {
    const n = nfa.node();
    try nfa.edge(n, a.begin, Charset.char(nfa.epsilon));
    try nfa.edge(a.final, n, Charset.char(nfa.epsilon));
    return .{ .begin = n, .final = n };
}

fn applyConcats(nfa: *Self, concats: usize, queue: *std.ArrayList(Slice)) !void {
    if (concats == 0) {
        return;
    }
    for (1..concats) |_| {
        const b = queue.pop();
        try queue.append(try nfa.concat(queue.pop(), b));
    }
}
