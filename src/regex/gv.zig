const std = @import("std");

pub const Node = struct {
    id: usize,
    begin: bool,
    final: bool,
};

pub const Edge = struct {
    from: usize,
    to: usize,
    char: u8,
};

const NodeIterator = struct {
    nodes: usize,
    begin: usize,
    final: []const bool,
    i: usize,

    pub fn next(it: *NodeIterator) ?Node {
        const i = it.i;
        if (i == it.nodes) {
            return null;
        }
        it.i += 1;
        return .{ .id = i, .begin = it.begin == i, .final = it.final[i] };
    }
};

const EdgeIterator = struct {
    s: []const []const usize,
    i: usize,
    m: usize,

    pub fn next(it: *EdgeIterator) ?Edge {
        if (it.i == it.s.len * it.m) {
            return null;
        }
        const a = it.i / it.m;
        const c = it.i % it.m;
        const b = it.s[a][c];
        it.i += 1;
        if (b >= it.s.len) {
            return it.next();
        }
        return .{ .from = a, .to = b, .char = @intCast(c) };
    }
};

pub fn print(g: anytype, w: anytype) !void {
    var nodes = nodeIterator(g);
    var edges = edgeIterator(g);
    try w.print("digraph {{\n", .{});
    try w.print("  0 [shape=point];\n", .{});
    while (nodes.next()) |n| {
        if (n.begin) {
            try w.print("  0 -> {};\n", .{n.id + 1});
        }
        try w.print("  {} [shape={s}];\n", .{ n.id + 1, if (n.final) "doublecircle" else "circle" });
    }
    try w.print("\n", .{});
    while (edges.next()) |e| {
        if (e.char >= ' ' and e.char < 127 and e.char != '"' and e.char != '\\') {
            try w.print("  {} -> {} [label=\"{c}\"];\n", .{ e.from + 1, e.to + 1, e.char });
        } else {
            try w.print("  {} -> {} [label=\"\\\\{o}\"];\n", .{ e.from + 1, e.to + 1, e.char });
        }
    }
    try w.print("}}\n", .{});
}

fn nodeIterator(s: anytype) NodeIterator {
    return .{ .nodes = s.nodes.len, .begin = 0, .final = s.final, .i = 0 };
}

fn edgeIterator(s: anytype) EdgeIterator {
    return .{ .s = s.nodes, .i = 0, .m = s.maxch };
}
