const std = @import("std");

// print any struct, that implements
// - fn getNode(self: Self, n: usize) ?struct { begin: bool, final: bool }
// - fn containsEdge(nfa: Self, a: usize, b: usize, c: u8) bool

pub fn print(g: anytype, w: anytype) !void {
    try w.print("digraph {{\n", .{});
    try w.print("  0 [shape=point];\n", .{});

    var nodes: usize = 0;
    while (g.getNode(nodes)) |n| {
        nodes += 1;
        if (n.begin) {
            try w.print("  0 -> {};\n", .{nodes});
        }
        try w.print("  {} [shape={s}];\n", .{ nodes, if (n.final) "doublecircle" else "circle" });
    }
    try w.print("\n", .{});

    for (0..nodes) |a| {
        for (0..nodes) |b| {
            for (0..256) |ch| {
                const c: u8 = @intCast(ch);
                if (g.containsEdge(a, b, c)) {
                    if (c >= ' ' and c < 127 and c != '"' and c != '\\') {
                        try w.print("  {} -> {} [label=\"{c}\"];\n", .{ a + 1, b + 1, c });
                    } else {
                        try w.print("  {} -> {} [label=\"\\\\{o}\"];\n", .{ a + 1, b + 1, c });
                    }
                }
            }
        }
    }
    try w.print("}}\n", .{});
}
