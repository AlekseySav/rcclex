const Self = @This();

m: u256,

pub fn init() Self {
    return .{ .m = 0 };
}

pub fn char(c: u8) Self {
    return .{ .m = bit(c) };
}

pub fn range(a: u8, b: u8) Self {
    var s = Self.init();
    for (a..b + 1) |c| {
        s.m |= bit(@intCast(c));
    }
    return s;
}

pub fn add(a: Self, b: Self) Self {
    return .{ .m = a.m | b.m };
}

pub fn xor(a: Self, b: Self) Self {
    return .{ .m = a.m ^ b.m };
}

pub fn int(a: Self, b: Self) Self {
    return .{ .m = a.m & b.m };
}

pub fn inv(a: Self) Self {
    return .{ .m = ~a.m };
}

pub fn contains(a: Self, c: u8) bool {
    return (a.m & bit(c)) != 0;
}

pub fn empty(a: Self) bool {
    return a.m == 0;
}

pub fn new(a: *Self) ?u8 {
    var it = a.inv().iterator();
    if (it.next()) |c| {
        a.* = a.add(char(c));
        return c;
    }
    return null;
}

pub fn iterator(a: Self) It {
    return .{ .m = a.m, .c = 0, .done = false };
}

pub fn maxChar(a: Self) usize {
    var c: u8 = 255;
    while (c > 0 and !a.contains(c)) {
        c -= 1;
    }
    return @as(usize, c) + 1;
}

fn bit(c: u8) u256 {
    return @as(u256, 1) << c;
}

const It = struct {
    m: u256,
    c: u8,
    done: bool,

    pub fn next(it: *It) ?u8 {
        while (!it.done and (it.m & bit(it.c)) == 0) {
            it.inc();
        }
        const r = if (it.done) null else it.c;
        it.inc();
        return r;
    }

    fn inc(it: *It) void {
        if (it.c == 255) {
            it.done = true;
            return;
        }
        it.c += 1;
    }
};
