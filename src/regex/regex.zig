const std = @import("std");
const gv = @import("gv.zig");

const Self = @This();

alloc: std.mem.Allocator,
nodes: []const []const usize,
final: []const bool,

pub fn deinit(s: Self) void {
    s.alloc.free(s.nodes);
    s.alloc.free(s.final);
}

const NodeIterator = struct {
    nodes: usize,
    begin: usize,
    final: []const bool,
    i: usize,

    pub fn next(it: *NodeIterator) ?gv.Node {
        const i = it.i;
        if (i == it.nodes) {
            return null;
        }
        it.i += 1;
        return .{ .id = i, .begin = it.begin == i, .final = it.final[i] };
    }
};

pub fn nodeIterator(nfa: Self) NodeIterator {
    return .{ .nodes = nfa.nodes.len, .begin = 0, .final = nfa.final, .i = 0 };
}

const EdgeIterator = struct {
    s: []const []const usize,
    i: usize,

    pub fn next(it: *EdgeIterator) ?gv.Edge {
        if (it.i == it.s.len * it.s[0].len) {
            return null;
        }
        const a = it.i / it.s[0].len;
        const c = it.i % it.s[0].len;
        const b = it.s[a][c];
        it.i += 1;
        if (b >= it.s.len) {
            return it.next();
        }
        return .{ .from = a, .to = b, .char = @intCast(c) };
    }
};

pub fn edgeIterator(nfa: Self) EdgeIterator {
    return .{ .s = nfa.nodes, .i = 0 };
}
