const std = @import("std");
const Charset = @import("re/charset.zig");

const Input = @This();

const ConfigError = error{
    BadCharsetRange,
};

const Range = struct {
    begin: []const u8,
    end: []const u8,

    fn parse(r: Range) !struct { a: u8, b: u8 } {
        return .{ .a = try parsebuf(r.begin), .b = try parsebuf(r.end) };
    }

    fn parsebuf(buf: []const u8) !u8 {
        if (buf.len == 0) {
            return ConfigError.BadCharsetRange;
        }
        if (buf[0] != '\\') {
            if (buf.len != 1) {
                return ConfigError.BadCharsetRange;
            }
            return buf[0];
        }
        var r: u8 = 0;
        for (buf[1..]) |c| {
            r = r * 8 + c - '0';
        }
        return r;
    }
};

config: struct {
    charset: struct {
        ranges: []const Range,
    },
},
tokens: []const struct {
    name: []const u8,
    pattern: []const u8,
},

pub fn charset(in: Input) !Charset {
    var s = Charset.init();
    for (in.config.charset.ranges) |c| {
        const p = try c.parse();
        _ = s.addRange(p.a, p.b);
    }
    return s;
}

pub fn pattern(in: Input, allocator: std.mem.Allocator) ![]const u8 {
    var len = in.tokens.len * 3 - 1;
    for (in.tokens) |t| {
        len += t.pattern.len;
    }
    const r = try allocator.alloc(u8, len);
    len = 0;
    for (in.tokens) |t| {
        if (len != 0) {
            r[len] = '|';
            len += 1;
        }
        r[len] = '(';
        len += 1;
        std.mem.copyForwards(u8, r[len..], t.pattern);
        len += t.pattern.len;
        r[len] = ')';
        len += 1;
    }
    return r;
}
