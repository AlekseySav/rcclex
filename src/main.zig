const std = @import("std");
const re = @import("regex");
const ymlz = @import("ymlz");
const Input = @import("input.zig");

const RcclexError = error{
    TooManyTokens,
};

pub fn makeExpr(in: Input, charset: *re.Charset, alloc: std.mem.Allocator) ![]const u8 {
    var len = in.tokens.len * 4 - 1;
    for (in.tokens) |t| {
        len += t.re.len;
    }
    const r = try alloc.alloc(u8, len);

    len = 0;
    for (in.tokens) |t| {
        if (len != 0) {
            r[len] = '|';
            len += 1;
        }
        r[len] = '(';
        len += 1;
        std.mem.copyForwards(u8, r[len..], t.re);
        len += t.re.len;
        r[len] = ')';
        len += 1;
        if (charset.new()) |c| {
            r[len] = c;
            len += 1;
        } else {
            return RcclexError.TooManyTokens;
        }
    }
    return r;
}

pub fn main() !void {
    var y = try ymlz.Ymlz(Input).init(std.heap.page_allocator);
    const input = try y.loadFile("/home/schet/src/rcclex/examples/1.yaml");
    defer y.deinit(input);

    var charset = try input.charset();
    const eps = charset.new();
    if (eps == null) {
        return RcclexError.TooManyTokens;
    }
    const pattern = try makeExpr(input, &charset, std.heap.page_allocator);
    defer std.heap.page_allocator.free(pattern);

    const r = try re.compile(std.heap.page_allocator, charset, pattern, eps.?);
    defer r.deinit();

    std.debug.print("read:\n", .{});
    for (0..r.nodes.len) |from| {
        std.debug.print("  - [", .{});
        for (input.config.output.minchar..input.config.output.maxchar + 1, 0..) |ch, i| {
            const c: u8 = @intCast(ch);
            if (i != 0) {
                std.debug.print(", ", .{});
            }
            if (charset.contains(c)) {
                std.debug.print("{}", .{r.nodes[from][c]});
            } else {
                std.debug.print("{}", .{r.nodes.len - 1});
            }
        }
        std.debug.print("]\n", .{});
    }

    // try re.gv.print(r, std.io.getStdOut().writer());
}
