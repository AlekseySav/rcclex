const std = @import("std");
const re = @import("regex");

pub fn main() !void {
    var lex = re.Lexer{
        .i = 0,
        .charset = re.Charset.range(' ', '~'),
        .pattern = "a(bc|b*\\*)",
    };

    var nfa = re.NFA.init(std.heap.page_allocator);
    defer nfa.deinit();
    try nfa.build(&lex);

    var nfa1 = re.NFA1.init(std.heap.page_allocator);
    defer nfa1.deinit();
    try nfa1.load(nfa);
    try nfa1.build();

    try re.gv.print(nfa1, std.io.getStdOut().writer());
}
