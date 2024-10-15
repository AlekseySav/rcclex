const std = @import("std");
const re = @import("regex");

const OutputError = error{
    AmbiguousToken,
};

pub const Token = struct {
    char: u8,
    id: []const u8,
};

pub const Config = struct {
    badToken: []const u8,
    charset: re.Charset,
    tokens: []const Token,
};

const Node = struct {
    used: bool,
    token: ?[]const u8,
    node: ?usize,
};

const Printer = struct {
    config: *const Config,
    r: *const re.Regex,
    nodes: []Node,
    count: usize,

    fn getNode(p: *Printer, n: usize) usize {
        if (p.nodes[n].node) |node| {
            return node;
        }
        p.nodes[n].node = p.count;
        p.count += 1;
        return p.nodes[n].node.?;
    }

    fn tokens(p: *Printer) !void {
        for (0..p.nodes.len) |n| {
            for (p.config.tokens) |t| {
                if (p.r.final[p.r.nodes[n][t.char]]) {
                    if (p.nodes[n].token != null) {
                        return OutputError.AmbiguousToken;
                    }
                    p.nodes[n].token = t.id;
                }
            }
        }
    }

    fn dfs(p: *Printer, w: anytype, n: usize) !void {
        if (p.nodes[n].used) {
            return;
        }
        p.nodes[n].used = true;
        try w.print("  {}:\n", .{p.getNode(n)});
        var cit = p.config.charset.iterator();
        while (cit.next()) |c| {
            if (p.config.charset.contains(c)) {
                if (p.r.hasChar(n, c)) {
                    try w.print("    {}: {}\n", .{ c, p.getNode(p.r.nodes[n][c]) });
                } else {
                    try w.print("    {}: -1\n", .{c});
                }
            }
        }
        cit = p.config.charset.iterator();
        while (cit.next()) |c| {
            if (p.r.hasChar(n, c)) {
                try p.dfs(w, p.r.nodes[n][c]);
            }
        }
    }

    fn print(p: *Printer, w: anytype) !void {
        try p.tokens();
        try w.print("nodes:\n", .{});
        try p.dfs(w, 0);
        try w.print("\ntoken:\n", .{});
        for (p.nodes) |n| {
            if (n.node) |node| {
                try w.print("  {}: '{s}'\n", .{ node, if (n.token) |t| t else p.config.badToken });
            }
        }
    }
};

pub fn printRegex(w: anytype, a: std.mem.Allocator, r: re.Regex, config: Config) !void {
    var p = Printer{
        .config = &config,
        .r = &r,
        .nodes = try a.alloc(Node, r.nodes.len),
        .count = 0,
    };
    @memset(p.nodes, .{ .used = false, .token = null, .node = null });
    return p.print(w);
}
