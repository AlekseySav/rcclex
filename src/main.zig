const std = @import("std");
const re = @import("regex");
const ymlz = @import("ymlz");
const Input = @import("input.zig");

pub fn main() !void {
    var y = try ymlz.Ymlz(Input).init(std.heap.page_allocator);
    const input = try y.loadFile("/home/schet/src/rcclex/examples/1.yaml");
    defer y.deinit(input);

    var charset = try input.charset();
    charset = charset.add(re.Charset.range(1, 2));
    const pattern = try input.joined(std.heap.page_allocator);

    const r = try re.compile(std.heap.page_allocator, charset, pattern);
    defer r.deinit();

    // match

    std.debug.print("read:\n", .{});
    for (0..r.nodes.len) |from| {
        std.debug.print("  - [", .{});
        for (input.config.output.minchar..input.config.output.maxchar + 1, 0..) |ch, i| {
            const c: u8 = @intCast(ch);
            if (i != 0) {
                std.debug.print(", ", .{});
            }
            if (charset.contains(c)) {
                std.debug.print("{}", .{r.nodes[from][c]});
            } else {
                std.debug.print("{}", .{r.nodes.len - 1});
            }
        }
        std.debug.print("]\n", .{});
    }

    try re.gv.print(r, std.io.getStdOut().writer());
}
