const std = @import("std");
const re = @import("regex");
const Self = @This();

const InputError = error{
    BadCharset,
    BadOutputFormat,
};

pub const OutputFormat = enum {
    Regex,
    Graphviz,
    Yaml,
};

config: struct {
    charset: struct {
        ranges: []const struct {
            begin: []const u8,
            end: []const u8,
        },
    },
    output: struct {
        format: []const u8,
        badtoken: []const u8,
    },
},

tokens: []const struct {
    id: []const u8,
    re: []const u8,
},

pub fn outputFormat(s: Self) !OutputFormat {
    const fmt = s.config.output.format;
    if (std.mem.eql(u8, fmt, "regex")) {
        return OutputFormat.Regex;
    }
    if (std.mem.eql(u8, fmt, "graphviz")) {
        return OutputFormat.Graphviz;
    }
    if (std.mem.eql(u8, fmt, "yaml")) {
        return OutputFormat.Yaml;
    }
    return InputError.BadOutputFormat;
}

pub fn charset(s: Self) !re.Charset {
    var c = re.Charset.init();
    for (s.config.charset.ranges) |r| {
        c = c.add(re.Charset.range(try range(r.begin), try range(r.end)));
    }
    return c;
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
