const std = @import("std");

pub const Charset = @import("compile/charset.zig");
pub const gv = @import("gv.zig");

pub const Regex = struct {
    alloc: std.mem.Allocator,
    nodes: []const []const usize,
    final: []const bool,

    pub fn deinit(s: Regex) void {
        for (s.nodes) |n| {
            s.alloc.free(n);
        }
        s.alloc.free(s.nodes);
        s.alloc.free(s.final);
    }

    pub fn hasChar(s: Regex, n: usize, c: usize) bool {
        return s.nodes[n][c] < s.nodes.len - 1;
    }

    pub fn getNode(s: Regex, n: usize) ?struct { begin: bool, final: bool } {
        if (n >= s.nodes.len - 1) {
            return null;
        }
        return .{ .begin = n == 0, .final = s.final[n] };
    }

    pub fn containsEdge(s: Regex, a: usize, b: usize, c: u8) bool {
        if (a >= s.nodes.len - 1 or b >= s.nodes.len or c >= s.nodes[a].len) {
            return false;
        }
        return s.nodes[a][c] == b;
    }
};

pub fn compile(alloc: std.mem.Allocator, charset: Charset, pattern: []const u8, eps: u8) !Regex {
    const Lexer = @import("compile/lexer.zig");
    const NFA = @import("compile/nfa.zig");
    const NFA1 = @import("compile/nfa1.zig");
    const DFA = @import("compile/dfa.zig");

    var lex = Lexer{
        .charset = charset,
        .pattern = pattern,
        .i = 0,
    };
    var nfa = NFA.init(alloc, eps);
    defer nfa.deinit();
    try nfa.build(&lex);
    var nfa1 = NFA1.init(alloc, eps);
    defer nfa1.deinit();
    try nfa1.load(nfa);
    try nfa1.build();
    var dfa = DFA.init(alloc, charset);
    defer dfa.deinit();
    try dfa.build(nfa1);
    try dfa.complete(charset);
    return .{
        .alloc = alloc,
        .nodes = try dfa.nodes.toOwnedSlice(),
        .final = try dfa.final.toOwnedSlice(),
    };
}
