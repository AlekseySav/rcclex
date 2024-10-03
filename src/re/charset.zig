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
        const c = it.c;
        it.c += 1;
        return c;
    }
};

c: [128]bool,

pub fn init() Charset {
    return .{ .c = [_]bool{false} ** 128 };
}

pub fn contains(s: Charset, c: u8) bool {
    return c < 128 and s.c[c];
}

pub fn imerge(s: *Charset, c: Charset) void {
    for (0..128) |i| {
        s.c[i] = s.c[i] or c.c[i];
    }
}

pub fn merge(s: Charset, c: Charset) Charset {
    var new = s;
    new.imerge(c);
    return new;
}

pub fn addRange(s: Charset, a: u8, b: u8) Charset {
    var new = s;
    for (a..b + 1) |c| {
        new.c[c] = true;
    }
    return new;
}

pub fn invert(s: Charset, superset: Charset) Charset {
    var new = s;
    for (0..128) |i| {
        new.c[i] = !s.c[i] and superset.c[i];
    }
    return new;
}

pub fn iadd(s: *Charset, c: u8) void {
    s.c[c] = true;
}

pub fn add(s: Charset, c: u8) Charset {
    var new = s;
    new.iadd(c);
    return new;
}

pub fn ipop(s: *Charset, c: u8) void {
    s.c[c] = false;
}

pub fn pop(s: Charset, c: u8) Charset {
    var new = s;
    new.ipop(c);
    return new;
}

pub fn empty(s: Charset) bool {
    for (s.c) |c| {
        if (c) {
            return false;
        }
    }
    return true;
}

pub fn iter(s: *const Charset) Iter {
    return .{ .s = s, .c = 0 };
}
