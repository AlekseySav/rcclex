const std = @import("std");
const re = @import("regex");
const ymlz = @import("ymlz");
const Input = @import("input.zig");
const output = @import("output.zig");

const InternalError = error{
    Leak,
};

pub fn main() !void {
    var a = std.heap.GeneralPurposeAllocator(.{}){};

    {
        var r = std.io.bufferedReader(std.io.getStdIn().reader());
        var y = try ymlz.Ymlz(Input).init(a.allocator());
        const input = try y.loadReader(r.reader().any());
        defer y.deinit(input);

        var w = std.io.bufferedWriter(std.io.getStdOut().writer());
        try run(input, a.allocator(), w.writer());
        try w.flush();
    }

    if (a.deinit() == .leak) {
        return InternalError.Leak;
    }
}

fn run(input: Input, a: std.mem.Allocator, w: anytype) !void {
    const outputFormat = try input.outputFormat();

    // get chatset & regex
    const originCharset = try input.charset();
    var charset = originCharset;
    const eps = try charset.new();
    const pattern = try makeExpr(input, &charset, a);
    defer a.free(pattern);
    if (outputFormat == .Regex) {
        return printRegex(pattern, w);
    }

    // compile regex
    const r = try re.compile(a, charset, pattern, eps);
    defer r.deinit();
    if (outputFormat == .Graphviz) {
        return re.print(r, w);
    }

    // output regex
    const tokens = try a.alloc(output.Token, input.tokens.len);
    defer a.free(tokens);
    charset = originCharset;
    _ = try charset.new();
    for (input.tokens, 0..) |t, i| {
        tokens[i] = .{
            .char = try charset.new(),
            .id = t.id,
        };
    }
    const config = output.Config{
        .badToken = input.config.output.badtoken,
        .charset = originCharset,
        .tokens = tokens,
    };
    try output.printRegex(w, a, r, config);
}

fn printRegex(pattern: []const u8, w: anytype) !void {
    for (pattern) |c| {
        if (std.ascii.isPrint(c)) {
            try w.print("{c}", .{c});
        } else {
            try w.print("\\{o}", .{c});
        }
    }
}

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
        r[len] = try charset.new();
        len += 1;
    }
    return r;
}
