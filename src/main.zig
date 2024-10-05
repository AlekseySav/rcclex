const std = @import("std");
const Charset = @import("regex/charset.zig");
const gv = @import("regex/gv.zig");
const Lexer = @import("regex/lexer.zig");
const NFA = @import("regex/nfa.zig");
const NFA1 = @import("regex/nfa-1.zig");

pub fn main() !void {
    var lex = Lexer{
        .i = 0,
        .charset = Charset.range(' ', '~'),
        .pattern = "a(bc|b*\\*)",
    };

    var nfa = NFA.init(std.heap.page_allocator);
    defer nfa.deinit();
    try nfa.build(&lex);

    var nfa1 = NFA1.init(std.heap.page_allocator);
    defer nfa1.deinit();
    try nfa1.load(nfa);
    try nfa1.build();

    try gv.print(nfa1, std.io.getStdOut().writer());
}
