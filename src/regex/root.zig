const std = @import("std");

pub const gv = @import("gv.zig");
pub const Charset = @import("charset.zig");

pub const Lexer = @import("lexer.zig");
pub const NFA = @import("nfa.zig");
pub const NFA1 = @import("nfa-1.zig");
pub const DFA = @import("dfa.zig");

pub const Config = struct {
    charset: Charset,
    pattern: []const u8,
};
