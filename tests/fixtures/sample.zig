const std = @import("std");

const Point = struct {
    x: f64,
    y: f64,
};

const Direction = enum {
    up,
    down,
    left,
    right,
};

const Result = union(enum) {
    ok: i32,
    err: []const u8,
};

const OpaqueType = opaque {};

pub fn add(a: i32, b: i32) i32 {
    return a + b;
}

fn greet(name: []const u8) []const u8 {
    return "Hello, " ++ name ++ "!";
}

pub fn magnitude(p: Point) f64 {
    return @sqrt(p.x * p.x + p.y * p.y);
}
