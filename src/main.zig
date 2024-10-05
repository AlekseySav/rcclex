const std = @import("std");
const re = @import("regex");

pub fn main() !void {
    var lex = re.Lexer{
        .i = 0,
        .charset = re.Charset.range('a', 'c'),
        .pattern = "a(bc|b*\\*)",
    };

    var nfa = re.NFA.init(std.heap.page_allocator);
    defer nfa.deinit();
    try nfa.build(&lex);

    var nfa1 = re.NFA1.init(std.heap.page_allocator);
    defer nfa1.deinit();
    try nfa1.load(nfa);
    try nfa1.build();

    var dfa = re.DFA.init(std.heap.page_allocator);
    defer dfa.deinit();
    try dfa.build(nfa1);
    try dfa.complete(lex.charset);

    try re.gv.print(dfa, std.io.getStdOut().writer());
}
