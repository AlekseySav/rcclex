const std = @import("std");
const Charset = @import("charset.zig");

const Self = @This();

const LexerError = error{
    BadChar,
    UnexpectedEnd,
    BadCharset,
};

const Token = struct {
    type: u8,
    charset: Charset,

    pub fn format(t: Token, comptime _: []const u8, _: std.fmt.FormatOptions, w: anytype) !void {
        if (t.type != '.') {
            return w.print("{c}", .{t.type});
        }
        var it = t.charset.iterator();
        try w.print("[", .{});
        while (it.next()) |c| {
            if (c >= ' ' and c < 127) {
                try w.print("{c}", .{c});
            } else {
                try w.print("\\{o}", .{c});
            }
        }
        try w.print("]", .{});
    }
};

charset: Charset,
pattern: []const u8,
i: usize,

pub fn token(lex: *Self) !?Token {
    const c = lex.char();
    return switch (c) {
        0 => null,
        '*', '+', '?', '|', '(', ')' => .{ .type = c, .charset = undefined },
        '[' => .{ .type = '.', .charset = try lex.chars() },
        '.' => .{ .type = '.', .charset = lex.charset },
        else => .{ .type = '.', .charset = try lex.onechar(c) },
    };
}

fn dec(lex: *Self) void {
    if (lex.i != lex.pattern.len) {
        lex.i -= 1;
    }
}

fn char(lex: *Self) u8 {
    if (lex.i == lex.pattern.len) {
        return 0;
    }
    lex.i += 1;
    return lex.pattern[lex.i - 1];
}

fn onechar(lex: *Self, c: u8) !Charset {
    if (c == 0) {
        return LexerError.UnexpectedEnd;
    }
    if (c == '\\') {
        return lex.escchar();
    }
    if (!lex.charset.contains(c)) {
        return LexerError.BadChar;
    }
    return Charset.char(c);
}

fn chars(lex: *Self) !Charset {
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
            var it = endset.iterator();
            const e = it.next();
            if (p == 0 or e == null or p >= e.? or it.next() != null) {
                return LexerError.BadCharset;
            }
            s = s.add(Charset.range(p, e.?));
        } else {
            s = s.add(try lex.onechar(c));
        }
        p = c;
        c = lex.char();
    }

    if (invert) {
        s = s.inv().int(lex.charset);
    }
    return s;
}

const charLF = Charset.char('\n');
const charHT = Charset.char('\t');
const charCR = Charset.char('\r');
const charD = Charset.range('0', '9');
const charW = Charset.range('a', 'z').add(Charset.range('A', 'Z'));

fn escchar(lex: *Self) !Charset {
    const c = lex.char();
    if (c == 0) {
        return LexerError.UnexpectedEnd;
    }
    return switch (c) {
        'n' => charLF,
        't' => charHT,
        'r' => charCR,
        'd' => charD,
        'D' => charD.inv().int(lex.charset),
        's' => Charset.char(' ').add(charCR).add(charLF).add(charHT),
        'S' => Charset.char(' ').add(charCR).add(charLF).add(charHT).inv().int(lex.charset),
        'w' => Charset.char('_').add(charD).add(charW),
        'W' => Charset.char('_').add(charD).add(charW).inv().int(lex.charset),
        '0', '1', '2', '3', '4', '5', '6', '7' => Charset.char(lex.atoi(c)),
        else => Charset.char(c),
    };
}

fn atoi(lex: *Self, ch: u8) u8 {
    var c = ch;
    var res: u8 = 0;
    while (c >= '0' and c <= '7') {
        res = res * 8 + c - '0';
        c = lex.char();
    }
    lex.dec();
    return res;
}
