const std = @import("std");
const re = @import("regex");

pub const Token = struct {
    char: u8,
    id: []const u8,
};

pub const Config = struct {
    minChar: usize,
    maxChar: usize,
    charset: re.Charset,
    tokens: []const Token,
};

const Node = struct {
    used: bool,
    token: usize,
    node: usize,
};

const Printer = struct {
    config: *const Config,
    r: *const re.Regex,
    nodes: []Node,

    fn printDfs(p: *Printer, w: anytype, n: usize) !void {
        if (p.used[n]) {
            return;
        }
        p.used[n] = true;

        _ = w;
    }
};

pub fn printRegex(w: anytype, a: std.mem.Allocator, r: re.Regex, config: Config) !void {
    var p = Printer{
        .config = config,
        .r = r,
        .used = try a.alloc(Node, r.nodes.len),
    };
    @memset(p.used, .{ .used = false, .token = 0, .node = 0 });
    return p.printDfs(w, 0);

    // try w.print("read:\n", .{});
    // for (0..r.nodes.len) |from| {
    //     try w.print("  - [", .{});
    //     for (config.minChar..config.maxChar + 1, 0..) |ch, i| {
    //         const c: u8 = @intCast(ch);
    //         if (i != 0) {
    //             try w.print(", ", .{});
    //         }
    //         if (config.charset.contains(c)) {
    //             try w.print("{}", .{r.nodes[from][c]});
    //         } else {
    //             try w.print("{}", .{r.nodes.len - 1});
    //         }
    //     }
    //     try w.print("]\n", .{});
    // }
}
