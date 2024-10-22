const std = @import("std");

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
    };
}

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
            gv.wprint("digraph {{\n", .{});
            for (0..a.size()) |i| {
                gv.wprint("  {} [shape=circle];\n", .{i + 1});
            }
            gv.wprint("  0 [shape=point];\n", .{});
            a.edges(null, used, .{ .impl = gv, .run = run });
            gv.wprint("}}\n", .{});
            if (gv.err) |e| {
                return e;
            }
        }

        fn run(cb: Callback, a: usize, b: usize, c: u8) void {
            const self: *Self = @ptrCast(@alignCast(cb.impl));
            self.gvedge(a, b, c);
        }

        fn gvedge(gv: *Self, a: usize, b: usize, c: u8) void {
            if (gv.first) {
                gv.w.print("  {} -> {};\n", .{ 0, a + 1 }) catch unreachable;
                gv.first = false;
            }
            if (c >= ' ' and c < 127 and c != '"' and c != '\\') {
                gv.wprint("  {} -> {} [label=\"{c}\"];\n", .{ a + 1, b + 1, c });
            } else {
                gv.wprint("  {} -> {} [label=\"\\\\{o}\"];\n", .{ a + 1, b + 1, c });
            }
        }

        fn wprint(gv: *Self, comptime fmt: []const u8, args: anytype) void {
            gv.w.print(fmt, args) catch |e| {
                gv.err = e;
            };
        }
    };
}
