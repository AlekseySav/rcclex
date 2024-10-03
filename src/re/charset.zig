const Charset = @This();

const Iter = struct {
    s: *const Charset,
    c: u8,

    pub fn next(it: *Iter) ?u8 {
        while (!it.s.contains(it.c)) {
            if (it.c >= 128) {
                return null;
            }
            it.c += 1;
        }
        return it.c;
    }
};

c: [128]bool,

pub fn init() Charset {
    return .{ .c = [_]bool{false} ** 128 };
}

pub fn contains(s: Charset, c: u8) bool {
    return c < 128 and s.c[c];
}

pub fn addRange(s: *Charset, a: u8, b: u8) *Charset {
    for (a..b + 1) |c| {
        s.c[c] = true;
    }
    return s;
}

pub fn add(s: *Charset, c: u8) *Charset {
    return s.addRange(c, c);
}

pub fn iter(s: Charset) Iter {
    return .{ .s = s, .c = 0 };
}
