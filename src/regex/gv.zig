const std = @import("std");
const Charset = @import("charset.zig");

pub const Node = struct {
    id: usize,
    begin: bool,
    final: bool,
};

pub const Edge = struct {
    from: usize,
    to: usize,
    charset: Charset,
};

pub fn print(g: anytype, w: anytype) !void {
    var nodes = g.nodeIterator();
    var edges = g.edgeIterator();
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
        var it = e.charset.iterator();
        while (it.next()) |c| {
            if (c >= ' ' and c < 127 and c != '"' and c != '\\') {
                try w.print("  {} -> {} [label=\"{c}\"];\n", .{ e.from + 1, e.to + 1, c });
            } else {
                try w.print("  {} -> {} [label=\"\\\\{o}\"];\n", .{ e.from + 1, e.to + 1, c });
            }
        }
    }
    try w.print("}}\n", .{});
}
