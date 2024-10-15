const std = @import("std");
const re = @import("regex");
const ymlz = @import("ymlz");
const Input = @import("input.zig");
const output = @import("output.zig");

const RcclexError = error{
    TooManyTokens,
};

fn makeExpr(in: Input, charset: *re.Charset, alloc: std.mem.Allocator) ![]const u8 {
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
    const a = std.heap.page_allocator;

    // load input
    var y = try ymlz.Ymlz(Input).init(std.heap.page_allocator);
    const input = try y.loadFile("/home/schet/src/rcclex/examples/1.yaml");
    defer y.deinit(input);

    // get chatset & regex
    const originCharset = try input.charset();
    var charset = originCharset;
    const eps = charset.new();
    if (eps == null) {
        return RcclexError.TooManyTokens;
    }
    const pattern = try makeExpr(input, &charset, a);
    defer a.free(pattern);

    // compile regex
    const r = try re.compile(a, charset, pattern, eps.?);
    defer r.deinit();

    // output regex
    const tokens = try a.alloc(output.Token, input.tokens.len);
    defer a.free(tokens);
    charset = originCharset;
    _ = charset.new();
    for (input.tokens, 0..) |t, i| {
        tokens[i] = .{
            .char = charset.new().?,
            .id = t.id,
        };
    }
    const config = output.Config{
        .badToken = input.config.output.badtoken,
        .charset = originCharset,
        .tokens = tokens,
    };
    try output.printRegex(std.io.getStdOut().writer(), a, r, config);

    // try re.gv.print(r, std.io.getStdOut().writer());
}
