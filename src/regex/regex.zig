const std = @import("std");
const common = @import("compile/common.zig");
const gv = @import("gv.zig");

const Self = @This();

alloc: std.mem.Allocator,
nodes: []const [common.MaxChar]usize,
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
    s: []const [common.MaxChar]usize,
    i: usize,

    pub fn next(it: *EdgeIterator) ?gv.Edge {
        if (it.i == it.s.len * common.MaxChar) {
            return null;
        }
        const a = it.i / common.MaxChar;
        const c = it.i % common.MaxChar;
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
