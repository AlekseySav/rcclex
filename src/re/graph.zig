const std = @import("std");
const Charset = @import("charset.zig");

const Graph = @This();

allocator: std.mem.Allocator,
begin: []bool,
final: []bool,
edges: []Charset,

pub fn init(allocator: std.mem.Allocator, count: usize) !Graph {
    const begin = try allocator.alloc(bool, count);
    const final = allocator.alloc(bool, count) catch |e| {
        allocator.free(begin);
        return e;
    };
    const edges = allocator.alloc(Charset, count * count) catch |e| {
        allocator.free(begin);
        allocator.free(final);
        return e;
    };
    @memset(begin, false);
    @memset(final, false);
    @memset(edges, Charset.init());
    return .{
        .allocator = allocator,
        .begin = begin,
        .final = final,
        .edges = edges,
    };
}

pub fn deinit(g: Graph) void {
    g.allocator.free(g.begin);
    g.allocator.free(g.final);
    g.allocator.free(g.edges);
}

pub fn node(g: *Graph, n: usize, begin: bool, final: bool) void {
    g.begin[n] = begin;
    g.final[n] = final;
}

pub fn edge(g: *Graph, from: usize, to: usize, s: Charset) void {
    const i = from * g.nodes() + to;
    g.edges[i] = g.edges[i].merge(s);
}

pub fn nodes(g: Graph) usize {
    return g.begin.len;
}

pub fn flush(g: Graph, w: std.io.AnyWriter) !void {
    try w.print("digraph {{\n", .{});
    try w.print("  0 [shape=point];\n", .{});
    for (g.final, 0..) |f, i| {
        try w.print("  {} [shape={s}];\n", .{ i + 1, if (f) "doublecircle" else "circle" });
    }
    try w.print("\n", .{});
    for (g.begin, 0..) |b, i| {
        if (b) {
            try w.print("  0 -> {};\n", .{i + 1});
        }
    }
    try w.print("\n", .{});
    for (0..g.nodes()) |a| {
        for (0..g.nodes()) |b| {
            var it = g.edges[a * g.nodes() + b].iter();
            while (it.next()) |c| {
                if (c >= ' ' and c < 127) {
                    try w.print("  {} -> {} [label=\"{c}\"];\n", .{ a + 1, b + 1, c });
                } else {
                    try w.print("  {} -> {} [label=\"\\\\{o}\"];\n", .{ a + 1, b + 1, c });
                }
            }
        }
    }
    try w.print("}}\n", .{});
}
