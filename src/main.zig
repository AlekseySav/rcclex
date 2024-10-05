const std = @import("std");
const re = @import("regex");

pub fn main() !void {
    const c = re.Config{
        .charset = re.Charset.range('a', 'c'),
        .pattern = "a(bc|b*\\*)",
    };

    const r = try re.compile(std.heap.page_allocator, c);
    defer r.deinit();

    try re.gv.print(r, std.io.getStdOut().writer());
}
