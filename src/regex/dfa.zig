const std = @import("std");
const Set = @import("zigset").Set;
const NFA1 = @import("nfa-1.zig");
const Charset = @import("charset.zig");
const gv = @import("gv.zig");
const common = @import("common.zig");

const Self = @This();

alloc: std.mem.Allocator,
nodes: std.ArrayList([common.MaxChar]usize),
final: std.ArrayList(bool),

pub fn init(a: std.mem.Allocator) Self {
    return .{
        .alloc = a,
        .nodes = std.ArrayList([common.MaxChar]usize).init(a),
        .final = std.ArrayList(bool).init(a),
    };
}

pub fn deinit(s: Self) void {
    s.nodes.deinit();
    s.final.deinit();
}

pub fn build(s: *Self, nfa: NFA1) !void {
    var queue = Queue(usize).init(s.alloc);
    var output = std.ArrayList(Set(usize)).init(s.alloc);
    defer {
        for (output.items) |*i| {
            i.deinit();
        }
        output.deinit();
        queue.deinit();
    }

    var beginState = Set(usize).init(s.alloc);
    _ = try beginState.add(nfa.begin);
    try queue.push(try s.node(nfa, &output, beginState));

    while (queue.pop()) |a| {
        for (0..common.MaxChar) |c| {
            const aState = output.items[a];

            var bState = Set(usize).init(s.alloc);
            var ait = aState.iterator();
            while (ait.next()) |n| {
                for (0..nfa.nodes.items.len) |i| {
                    if (nfa.nodes.items[n.*].items[i].contains(@intCast(c))) {
                        _ = try bState.add(i);
                    }
                }
            }
            if (bState.isEmpty()) {
                bState.deinit();
                continue;
            }

            const b = find(output, bState);
            if (b) |n| {
                bState.deinit();
                s.nodes.items[a][c] = n;
                continue;
            }
            const n = try s.node(nfa, &output, bState);
            try queue.push(n);
            s.nodes.items[a][c] = n;
        }
    }
}

pub fn complete(s: *Self, c: Charset) !void {
    const r = s.nodes.items.len;
    try s.nodes.append([_]usize{std.math.maxInt(usize)} ** common.MaxChar);
    try s.final.append(false);

    for (s.nodes.items) |*a| {
        for (a, 0..) |*b, i| {
            if (c.contains(@intCast(i)) and b.* > r) {
                b.* = r;
            }
        }
    }
}

fn find(list: std.ArrayList(Set(usize)), v: Set(usize)) ?usize {
    for (list.items, 0..) |s, i| {
        if (s.eql(v)) {
            return i;
        }
    }
    return null;
}

fn node(s: *Self, nfa: NFA1, output: *std.ArrayList(Set(usize)), state: Set(usize)) !usize {
    const r = s.nodes.items.len;
    try s.nodes.append([_]usize{std.math.maxInt(usize)} ** common.MaxChar);
    try s.final.append(false);

    try output.append(state);
    var it = state.iterator();
    while (it.next()) |n| {
        if (nfa.final.items[n.*] == true) {
            s.final.items[n.*] = true;
        }
    }
    return r;
}

fn Queue(comptime T: type) type {
    return struct {
        const Q = @This();
        const Impl = std.fifo.LinearFifo(T, std.fifo.LinearFifoBufferType.Dynamic);

        impl: Impl,

        pub fn init(a: std.mem.Allocator) Q {
            return .{ .impl = Impl.init(a) };
        }

        pub fn deinit(q: Q) void {
            q.impl.deinit();
        }

        pub fn push(q: *Q, v: T) !void {
            const buf = [_]T{v};
            try q.impl.write(&buf);
        }

        pub fn pop(q: *Q) ?T {
            return q.impl.readItem();
        }
    };
}

pub fn nodeIterator(nfa: Self) NFA1.NodeIterator {
    return .{ .nodes = nfa.nodes.items.len, .begin = 0, .final = nfa.final.items, .i = 0 };
}

pub const EdgeIterator = struct {
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
        return .{ .from = a, .to = b, .charset = Charset.char(@intCast(c)) };
    }
};

pub fn edgeIterator(nfa: Self) EdgeIterator {
    return .{ .s = nfa.nodes.items, .i = 0 };
}
