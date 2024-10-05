const std = @import("std");
const Charset = @import("charset.zig");
const NFA = @import("nfa.zig");
const gv = @import("gv.zig");

const Self = @This();

alloc: std.mem.Allocator,
nodes: std.ArrayList(std.ArrayList(Charset)),
final: std.ArrayList(bool),
begin: usize,

pub fn init(a: std.mem.Allocator) Self {
    return .{
        .alloc = a,
        .nodes = std.ArrayList(std.ArrayList(Charset)).init(a),
        .final = std.ArrayList(bool).init(a),
        .begin = undefined,
    };
}

pub fn deinit(s: Self) void {
    for (s.nodes.items) |i| {
        i.deinit();
    }
    s.nodes.deinit();
}

pub fn load(s: *Self, nfa: NFA) !void {
    try s.final.appendNTimes(false, nfa.nodes);
    try s.nodes.appendNTimes(std.ArrayList(Charset).init(s.alloc), nfa.nodes);
    for (s.nodes.items) |*i| {
        try i.appendNTimes(Charset.init(), nfa.nodes);
    }
    for (nfa.edges.items) |e| {
        s.nodes.items[e.a].items[e.b] = e.c;
    }
    s.begin = nfa.slice.begin;
    s.final.items[nfa.slice.final] = true;
}

pub fn build(nfa: *Self) !void {
    const used = try nfa.alloc.alloc(bool, nfa.nodes.items.len);
    defer nfa.alloc.free(used);
    for (0..nfa.nodes.items.len) |i| {
        @memset(used, false);
        nfa.epsilonDfs(used, i, i);
    }
    for (nfa.nodes.items) |*i| {
        for (i.items) |*c| {
            if (c.contains(0)) {
                c.* = c.xor(Charset.char(0));
            }
        }
    }

    @memset(used, false);
    nfa.visitDfs(used, nfa.begin);
    const nodes = nfa.nodes.items.len;
    var last = nodes - 1;
    for (0..nodes) |n| {
        if (!used[nodes - n - 1]) {
            nfa.swapNodes(last, nodes - n - 1);
            last -= 1;
        }
    }
    nfa.nodes.shrinkAndFree(last + 1);
    for (nfa.nodes.items) |*i| {
        i.shrinkAndFree(last + 1);
    }
}

fn epsilonDfs(nfa: *Self, used: []bool, p: usize, n: usize) void {
    used[n] = true;
    for (nfa.nodes.items[n].items, 0..) |c, i| {
        if (used[i] or !c.contains(0)) {
            continue;
        }
        epsilonDfs(nfa, used, p, i);
        if (nfa.final.items[i]) {
            nfa.final.items[n] = true;
        }
    }
    for (nfa.nodes.items[n].items, 0..) |c, i| {
        nfa.nodes.items[p].items[i] = nfa.nodes.items[p].items[i].add(c);
    }
}

fn visitDfs(nfa: *Self, used: []bool, n: usize) void {
    used[n] = true;
    for (nfa.nodes.items[n].items, 0..) |c, i| {
        if (used[i] or c.empty()) {
            continue;
        }
        nfa.visitDfs(used, i);
    }
}

fn swap(comptime T: type) type {
    return struct {
        pub fn ptr(a: *T, b: *T) void {
            const c = a.*;
            a.* = b.*;
            b.* = c;
        }
    };
}

fn swapNodes(nfa: *Self, a: usize, b: usize) void {
    swap(bool).ptr(&nfa.final.items[a], &nfa.final.items[b]);
    if (nfa.begin == a) {
        nfa.begin = b;
    } else if (nfa.begin == b) {
        nfa.begin = a;
    }
    swap(std.ArrayList(Charset)).ptr(&nfa.nodes.items[a], &nfa.nodes.items[b]);
    for (0..nfa.nodes.items.len) |i| {
        swap(Charset).ptr(&nfa.nodes.items[i].items[a], &nfa.nodes.items[i].items[b]);
    }
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
    return .{ .nodes = nfa.nodes.items.len, .begin = nfa.begin, .final = nfa.final.items, .i = 0 };
}

const EdgeIterator = struct {
    s: []const std.ArrayList(Charset),
    i: usize,

    pub fn next(it: *EdgeIterator) ?gv.Edge {
        if (it.i == it.s.len * it.s.len) {
            return null;
        }
        const a = it.i / it.s.len;
        const b = it.i % it.s.len;
        const c = it.s[a].items[b];
        it.i += 1;
        if (c.empty()) {
            return it.next();
        }
        return .{ .from = a, .to = b, .charset = c };
    }
};

pub fn edgeIterator(nfa: Self) EdgeIterator {
    return .{ .s = nfa.nodes.items, .i = 0 };
}