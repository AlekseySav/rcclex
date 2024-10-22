const std = @import("std");
const Set = @import("zigset").Set;

/// Common interface for NFA, DFA
/// used for testing, printing
pub fn Automation(A: type) type {
    return struct {
        const Self = @This();
        pub const Callback = struct {
            run: *const fn (cb: Callback, a: usize, b: usize, c: u8) void,
            impl: *anyopaque,
        };

        allocator: std.mem.Allocator,
        a: A,

        pub fn init(alloc: std.mem.Allocator, a: A) Self {
            return .{ .allocator = alloc, .a = a };
        }

        pub fn graphviz(a: Self, w: anytype) !void {
            var gv = Graphviz(A, @TypeOf(w)).init(a.allocator, w);
            try gv.print(a.a);
        }

        pub fn traverse(a: Self, s: []const u8) !?usize {
            var table = Table(A).init(a.allocator);
            defer table.deinit();
            try table.load(a.a);
            return table.traverse(table.begin, s);
        }
    };
}

// implements graphviz formatting
fn Graphviz(A: type, W: type) type {
    return struct {
        const Callback = Automation(A).Callback;
        const Self = @This();
        const PrintReturnType = @typeInfo(@TypeOf(W.print)).Fn.return_type.?;
        const ErrorType = @typeInfo(PrintReturnType).ErrorUnion.error_set;

        allocator: std.mem.Allocator,
        w: W,
        first: bool,
        err: ?ErrorType,

        fn init(a: std.mem.Allocator, w: W) Self {
            return .{ .allocator = a, .w = w, .first = true, .err = null };
        }

        fn print(gv: *Self, a: A) !void {
            const used = try gv.allocator.alloc(bool, a.size());
            defer gv.allocator.free(used);
            @memset(used, false);
            try gv.w.print("digraph {{\n", .{});
            for (0..a.size()) |i| {
                try gv.w.print("  {} [shape=circle];\n", .{i + 1});
            }
            try gv.w.print("  0 [shape=point];\n", .{});
            a.edges(null, used, .{ .impl = gv, .run = callback });
            if (gv.err) |e| {
                return e;
            }
            try gv.w.print("}}\n", .{});
        }

        fn callback(cb: Callback, a: usize, b: usize, c: u8) void {
            const s: *Self = @ptrCast(@alignCast(cb.impl));
            if (s.err) |_| {
                return;
            }
            s.gvedge(a, b, c) catch |e| {
                s.err = e;
            };
        }

        fn gvedge(gv: *Self, a: usize, b: usize, c: u8) ErrorType!void {
            if (gv.first) {
                try gv.w.print("  {} -> {};\n", .{ 0, a + 1 });
                gv.first = false;
            }
            if (c >= ' ' and c < 127 and c != '"' and c != '\\') {
                try gv.w.print("  {} -> {} [label=\"{c}\"];\n", .{ a + 1, b + 1, c });
            } else {
                try gv.w.print("  {} -> {} [label=\"\\\\{o}\"];\n", .{ a + 1, b + 1, c });
            }
        }
    };
}

// implements pattern lookups
fn Table(A: type) type {
    return struct {
        const Callback = Automation(A).Callback;
        const Self = @This();
        const Node = std.AutoHashMap(u8, Set(usize));

        alloc: std.mem.Allocator,
        nodes: std.ArrayList(Node),
        begin: usize,
        first: bool,
        err: ?std.mem.Allocator.Error,

        fn init(a: std.mem.Allocator) Self {
            return .{
                .alloc = a,
                .nodes = std.ArrayList(Node).init(a),
                .err = null,
                .begin = 0,
                .first = true,
            };
        }

        fn deinit(t: *Self) void {
            for (t.nodes.items) |*node| {
                var nit = node.valueIterator();
                while (nit.next()) |set| {
                    set.deinit();
                }
                node.deinit();
            }
            t.nodes.deinit();
        }

        fn load(t: *Self, a: A) !void {
            const used = try t.alloc.alloc(bool, a.size());
            defer t.alloc.free(used);
            @memset(used, false);
            for (0..a.size()) |_| {
                try t.nodes.append(Node.init(t.alloc));
            }
            a.edges(null, used, .{ .impl = t, .run = run });
            if (t.err) |e| {
                return e;
            }
        }

        fn run(cb: Callback, a: usize, b: usize, c: u8) void {
            const s: *Self = @ptrCast(@alignCast(cb.impl));
            if (s.err) |_| {
                return;
            }
            s.add(a, b, c) catch |e| {
                s.err = e;
            };
        }

        fn add(t: *Self, a: usize, b: usize, c: u8) !void {
            if (t.first) {
                t.first = false;
                t.begin = a;
            }
            var set = try t.nodes.items[a].getOrPutValue(c, Set(usize).init(t.alloc));
            _ = try set.value_ptr.add(b);
        }

        fn traverse(t: Self, n: usize, s: []const u8) ?usize {
            if (s.len == 0) {
                return n;
            }
            const node = t.nodes.items[n];
            if (node.get(s[0])) |set| {
                var it = set.iterator();
                while (it.next()) |next| {
                    if (t.traverse(next.*, s[1..])) |r| {
                        return r;
                    }
                }
            }
            return null;
        }
    };
}
