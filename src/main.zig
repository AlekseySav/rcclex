const std = @import("std");
const Ymlz = @import("ymlz").Ymlz;

const Input = @import("input.zig");
const Charset = @import("re/charset.zig");

const UsageError = error{
    BadArgs,
};

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer {
        if (gpa.deinit() == .leak) {
            @panic("leaks detected");
        }
    }

    {
        var config: ?[]const u8 = null;
        var args = try std.process.ArgIterator.initWithAllocator(gpa.allocator());
        defer args.deinit();
        _ = args.skip();
        while (args.next()) |arg| {
            if (config != null) {
                return UsageError.BadArgs;
            }
            config = arg;
        }
        if (config == null) {
            return UsageError.BadArgs;
        }

        var ymlz = try Ymlz(Input).init(gpa.allocator());
        const yaml_path = try std.fs.cwd().realpathAlloc(gpa.allocator(), config.?);
        defer gpa.allocator().free(yaml_path);

        const input = try ymlz.loadFile(yaml_path);
        defer ymlz.deinit(input);

        const pattern = try input.pattern(gpa.allocator());
        defer gpa.allocator().free(pattern);

        std.debug.print("{} {} {s}\n", .{ input, try input.charset(), pattern });
    }
}
