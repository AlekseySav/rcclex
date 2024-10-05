const std = @import("std");
const common = @import("common.zig");
const NFA1 = @import("nfa-1.zig");
const DFA = @import("dfa.zig");

const Self = @This();

alloc: std.mem.Allocator,
nodes: []const [common.MaxChar]usize,
final: []const bool,

pub fn deinit(s: Self) void {
    s.alloc.free(s.nodes);
    s.alloc.free(s.final);
}

pub fn nodeIterator(nfa: Self) NFA1.NodeIterator {
    return .{ .nodes = nfa.nodes.len, .begin = 0, .final = nfa.final, .i = 0 };
}

pub fn edgeIterator(nfa: Self) DFA.EdgeIterator {
    return .{ .s = nfa.nodes, .i = 0 };
}
