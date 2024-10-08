const std = @import("std");
const re = @import("regex");
const Self = @This();

const InputError = error{
    BadCharset,
};

config: struct {
    charset: struct {
        ranges: []const struct {
            begin: []const u8,
            end: []const u8,
        },
    },
    output: struct {
        minchar: usize,
        maxchar: usize,
    },
},

tokens: []const struct {
    id: []const u8,
    re: []const u8,
},

pub fn charset(s: Self) !re.Charset {
    var c = re.Charset.init();
    for (s.config.charset.ranges) |r| {
        c = c.add(re.Charset.range(try range(r.begin), try range(r.end)));
    }
    return c;
}

pub fn joined(in: Self, alloc: std.mem.Allocator) ![]const u8 {
    var len = in.tokens.len * 4 - 1;
    for (in.tokens) |t| {
        len += t.re.len;
    }
    const r = try alloc.alloc(u8, len);
    len = 0;
    for (in.tokens, 1..) |t, i| {
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
        r[len] = @intCast(i);
        len += 1;
    }
    return r;
}
fn range(p: []const u8) !u8 {
    var r: u8 = 0;
    var esc = false;
    var end = false;
    for (p) |c| {
        if (c == '\\') {
            esc = true;
            continue;
        }
        if (!esc) {
            if (r != 0) {
                return InputError.BadCharset;
            }
            r = c;
            continue;
        }
        r = r * 8 + c - '0';
        if (c < '0' or c > '7') {
            return InputError.BadCharset;
        }
        end = true;
    }
    if (esc and !end) {
        return InputError.BadCharset;
    }
    return r;
}
