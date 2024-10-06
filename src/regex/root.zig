const std = @import("std");
const Lexer = @import("compile/lexer.zig");
const NFA = @import("compile/nfa.zig");
const NFA1 = @import("compile/nfa-1.zig");
const DFA = @import("compile/dfa.zig");

pub const Charset = @import("compile/charset.zig");
pub const Regex = @import("regex.zig");
pub const gv = @import("gv.zig");

pub const Config = struct {
    charset: Charset,
    pattern: []const u8,
};

pub fn compile(alloc: std.mem.Allocator, c: Config) !Regex {
    var lex = Lexer{
        .charset = c.charset,
        .pattern = c.pattern,
        .i = 0,
    };
    var nfa = NFA.init(alloc);
    defer nfa.deinit();
    try nfa.build(&lex);
    var nfa1 = NFA1.init(alloc);
    defer nfa1.deinit();
    try nfa1.load(nfa);
    try nfa1.build();
    var dfa = DFA.init(alloc, c.charset);
    defer dfa.deinit();
    try dfa.build(nfa1);
    try dfa.complete(c.charset);
    return .{
        .alloc = alloc,
        .nodes = try dfa.nodes.toOwnedSlice(),
        .final = try dfa.final.toOwnedSlice(),
        .maxch = dfa.maxChar,
    };
}
