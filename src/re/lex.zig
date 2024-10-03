const std = @import("std");
const Charset = @import("charset.zig");

const Lexer = @This();

const ParseError = error{
    BadChar,
    UnexpectedEnd,
    BadCharset,
};

const Token = struct {
    type: u8,
    charset: Charset,

    pub fn format(t: Token, comptime _: []const u8, _: std.fmt.FormatOptions, writer: anytype) !void {
        if (t.type != '.') {
            return writer.print("{c}", .{t.type});
        }
        var it = t.charset.iter();
        try writer.print("[", .{});
        while (it.next()) |c| {
            try writer.print("{c}", .{c});
        }
        try writer.print("]", .{});
    }
};

charset: Charset,
pattern: []const u8,
i: u32,

pub fn token(lex: *Lexer) !?Token {
    const c = lex.char();
    return switch (c) {
        0 => null,
        '*', '+', '?', '|', '(', ')' => .{ .type = c, .charset = undefined },
        '[' => .{ .type = '.', .charset = try lex.chars() },
        '.' => .{ .type = '.', .charset = lex.charset },
        else => .{ .type = '.', .charset = try lex.onechar(c) },
    };
}

fn dec(lex: *Lexer) void {
    if (lex.i != lex.pattern.len) {
        lex.i -= 1;
    }
}

fn char(lex: *Lexer) u8 {
    if (lex.i == lex.pattern.len) {
        return 0;
    }
    lex.i += 1;
    return lex.pattern[lex.i - 1];
}

fn onechar(lex: *Lexer, c: u8) !Charset {
    if (c == 0) {
        return ParseError.UnexpectedEnd;
    }
    if (c == '\\') {
        return lex.escchar();
    }
    if (!lex.charset.contains(c)) {
        return ParseError.BadChar;
    }
    return Charset.init().add(c);
}

fn chars(lex: *Lexer) !Charset {
    var s = Charset.init();
    var p: u8 = 0;

    var c = lex.char();
    var invert = false;
    if (c == '^') {
        invert = true;
        c = lex.char();
    }

    while (c != ']') {
        if (c == '-') {
            const endset = try lex.onechar(lex.char());
            var it = endset.iter();
            const e = it.next();
            if (p == 0 or e == null or p >= e.? or it.next() != null) {
                return ParseError.BadCharset;
            }
            s = s.addRange(p, e.?);
        } else {
            s = s.merge(try lex.onechar(c));
        }
        p = c;
        c = lex.char();
    }

    if (invert) {
        s = s.invert(lex.charset);
    }
    return s;
}

fn escchar(lex: *Lexer) !Charset {
    const c = lex.char();
    if (c == 0) {
        return ParseError.UnexpectedEnd;
    }
    return switch (c) {
        't' => Charset.init().add('\t'),
        'n' => Charset.init().add('\n'),
        'r' => Charset.init().add('\r'),
        'd' => Charset.init().addRange('0', '9'),
        'D' => Charset.init().addRange('0', '9').invert(lex.charset),
        's' => Charset.init().add('\r').add('\t').add('\n').add(' '),
        'S' => Charset.init().add('\r').add('\t').add('\n').add(' ').invert(lex.charset),
        'w' => Charset.init().addRange('0', '9').addRange('A', 'Z').addRange('a', 'z').add('_'),
        'W' => Charset.init().addRange('0', '9').addRange('A', 'Z').addRange('a', 'z').add('_').invert(lex.charset),
        '0', '1', '2', '3', '4', '5', '6', '7' => Charset.init().add(lex.atoi(c)),
        else => Charset.init().add(c),
    };
}

fn atoi(lex: *Lexer, ch: u8) u8 {
    var c = ch;
    var res: u8 = 0;
    while (c >= '0' and c <= '7') {
        res = res * 8 + c - '0';
        c = lex.char();
    }
    lex.dec();
    return res;
}
