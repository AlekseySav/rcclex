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

        allocator: std.mem.Allocator,
        w: W,
        first: bool,

        fn init(a: std.mem.Allocator, w: W) Self {
            return .{ .allocator = a, .w = w, .first = true };
        }

        fn print(gv: *Self, a: A) !void {
            const used = try gv.allocator.alloc(bool, a.size());
            defer gv.allocator.free(used);
            @memset(used, false);
            std.debug.print("digraph {{\n", .{});
            std.debug.print("  0 [shape=point];\n", .{});
            a.edges(null, used, .{ .impl = gv, .run = run });
            std.debug.print("}}\n", .{});
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
                gv.w.print("  {} -> {} [label=\"{c}\"];\n", .{ a + 1, b + 1, c }) catch unreachable;
            } else {
                gv.w.print("  {} -> {} [label=\"\\\\{o}\"];\n", .{ a + 1, b + 1, c }) catch unreachable;
            }
        }
    };
}
