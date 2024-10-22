const std = @import("std");

pub const Charset = @import("compile/charset.zig");
const Automation = @import("compile/automation.zig").Automation;

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

    pub fn size(s: Regex) usize {
        return s.nodes.len - 1;
    }

    pub fn edges(s: Regex, n: ?usize, used: []bool, cb: Automation(Regex).Callback) void {
        const a: usize = if (n) |nd| nd else 0;
        used[a] = true;
        for (s.nodes[a], 0..) |b, c| {
            if (b < s.nodes.len - 1) {
                cb.run(cb, a, b, @intCast(c));
                if (!used[b]) {
                    s.edges(b, used, cb);
                }
            }
        }
    }
};

pub fn print(re: Regex, w: anytype) !void {
    return Automation(Regex).init(re.alloc, re).graphviz(w);
}

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
