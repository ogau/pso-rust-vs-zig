const std = @import("std");

const pso = @import("pso.zig");
const PSO = pso.PSO;
const Range = pso.Range;

fn pow2(x: f64) f64 {
    return x * x;
}

fn dixonprice(x: []const f64) f64 {
    var sum = pow2(x[0] - 1.0);

    for (1..x.len) |i| {
        sum += @as(f64, @floatFromInt(i + 1)) * pow2(2.0 * pow2(x[i]) - x[i - 1]);
    }

    return sum;
}

fn sphere(solution: []const f64) f64 {
    const x = solution[0];
    const y = solution[1];
    return pow2(x - 3.14) + pow2(y - 2.72) + std.math.sin(3 * x + 1.41) + std.math.sin(4 * y - 1.73);
}

pub fn main() !void {
    var start = std.time.milliTimestamp();

    const DIMS = 200;

    const n_particles = 500;

    const params = .{
        .w = 0.99,
        .init_speed = 0.01,

        .n_particles = n_particles,
        .ranges = [_]Range{.{ .min = -10.0, .max = 10.0 }} ** DIMS,
    };
    var opt = PSO(DIMS).init(std.heap.page_allocator, params);
    defer opt.deinit();

    var buf = try std.ArrayList(f64).initCapacity(std.heap.page_allocator, n_particles);
    defer buf.deinit();

    const total_num_generations = 20000;
    for (0..total_num_generations) |num_gen| {
        buf.clearRetainingCapacity();

        var iter = opt.ask();
        while (iter.next()) |solution| {
            const eval = dixonprice(solution);
            try buf.append(eval);
        }

        // for (buf.items) |eval| {
        //     std.debug.print("{} {d:.2}\n", .{ num_gen, eval });
        // }

        const best_obj = opt.tell(buf.items);
        if ((num_gen + 1) % 250 == 0) {
            std.debug.print("{} {d:.4}\n", .{ num_gen, best_obj });
        }
    }

    var end = std.time.milliTimestamp();
    std.debug.print("{}ms\n", .{end - start});
}
