const std = @import("std");
const Allocator = std.mem.Allocator;
const DefaultPrng = std.rand.DefaultPrng;
const Random = std.rand.Random;

pub const Range = struct {
    min: f64,
    max: f64,
    fn randomize(self: Range, r: Random) f64 {
        return r.float(f64) * (self.max - self.min) + self.min;
    }
};

pub fn PSO(comptime solution_len: usize) type {
    return struct {
        pub const Params = struct {
            n_particles: usize = 20,
            ranges: [solution_len]Range,
            c1: f64 = 0.1,
            c2: f64 = 0.1,
            w: f64 = 0.8,
            init_speed: f64 = 0.1,
        };

        const Self = @This();

        params: Params,

        positions: DenseMatrix(f64),
        velocities: DenseMatrix(f64),

        pers_best_positions: DenseMatrix(f64),
        pers_best_obj_list: Vec(f64),

        rng: RomuDuoJr,

        pub fn init(allocator: Allocator, params: Params) Self {
            const shape = .{ .rows = params.n_particles, .cols = solution_len };

            const velocities = DenseMatrix(f64).init(allocator, shape, undefined);
            const positions = DenseMatrix(f64).init(allocator, shape, undefined);
            const pers_best_positions = DenseMatrix(f64).init(allocator, shape, undefined);
            const pers_best_obj_list = Vec(f64).init(allocator, params.n_particles, std.math.inf(f64));

            const seed = m: {
                var seed: u64 = undefined;
                std.os.getrandom(std.mem.asBytes(&seed)) catch @panic("getrandom panic");
                break :m seed;
            };

            var r = RomuDuoJr.init(seed);
            const rng = r.random();

            // init positions
            for (0..params.n_particles) |i| {
                const pos_row = positions.row_view(i);
                for (0..solution_len) |j| {
                    const range = params.ranges[j];
                    pos_row[j] = range.randomize(rng);
                }
            }

            // copy positions
            @memcpy(pers_best_positions.data, positions.data);

            // init velocities
            for (velocities.data) |*v| {
                v.* = rng.floatNorm(f64) * params.init_speed;
            }

            return Self{
                .positions = positions,
                .velocities = velocities,
                .params = params,
                .rng = r,
                .pers_best_positions = pers_best_positions,
                .pers_best_obj_list = pers_best_obj_list,
            };
        }

        const IndividsIterator = struct {
            data: DenseMatrix(f64),
            current_index: usize,

            pub fn next(self: *IndividsIterator) ?[]const f64 {
                const index = self.current_index;
                if (index < self.data.shape.rows) {
                    self.current_index += 1;
                    return self.data.row_view(index);
                } else {
                    return null;
                }
            }
        };

        pub fn ask(self: *Self) IndividsIterator {
            return IndividsIterator{
                .data = self.positions,
                .current_index = 0,
            };
        }

        pub fn tell(self: *Self, objectives: []const f64) f64 {
            // replace best individs
            for (0.., objectives) |i, obj| {
                const pers_best_obj = &self.pers_best_obj_list.slice[i];

                if (obj < pers_best_obj.*) {
                    pers_best_obj.* = obj;

                    const pers_best_pos = self.pers_best_positions.row_view(i);
                    const pos = self.positions.row_view(i);
                    @memcpy(pers_best_pos, pos);
                }
            }

            // argmin
            const glob_best_ind = self.pers_best_obj_list.argmin();

            const w = self.params.w;
            const c1 = self.params.c1;
            const c2 = self.params.c2;

            const glob_best_pos = self.pers_best_positions.row_view(glob_best_ind);
            const glob_best_obj = self.pers_best_obj_list.slice[glob_best_ind];

            for (0..self.params.n_particles) |i| {
                const pos_row = self.positions.row_view(i);
                const vel_row = self.velocities.row_view(i);
                const pers_best_pos_row = self.pers_best_positions.row_view(i);
                for (0..solution_len) |d| {
                    const r1 = self.rng.next_f64();
                    const r2 = self.rng.next_f64();

                    vel_row[d] = (w * vel_row[d] +
                        c1 * r1 * (pers_best_pos_row[d] - pos_row[d]) +
                        c2 * r2 * (glob_best_pos[d] - pos_row[d]));
                    pos_row[d] += vel_row[d];
                }
            }

            return glob_best_obj;
        }

        pub fn deinit(self: Self) void {
            self.positions.deinit();
            self.velocities.deinit();
            self.pers_best_obj_list.deinit();
            self.pers_best_positions.deinit();
        }
    };
}

fn Vec(comptime T: type) type {
    return struct {
        const Self = @This();

        slice: []T,
        allocator: Allocator,

        fn init(allocator: Allocator, n: usize, elem: T) Self {
            const new_mem = allocator.alloc(T, n) catch @panic("alloc panic");
            @memset(new_mem, elem);
            return Self{ .slice = new_mem, .allocator = allocator };
        }

        fn argmin(self: Self) usize {
            var best_ind: usize = 0;
            var min = self.slice[0];
            for (1.., self.slice[1..]) |i, v| {
                if (v < min) {
                    min = v;
                    best_ind = i;
                }
            }
            return best_ind;
        }

        fn deinit(self: Self) void {
            self.allocator.free(self.slice);
        }
    };
}

fn DenseMatrix(comptime T: type) type {
    return struct {
        const Self = @This();

        const Shape = struct {
            rows: usize,
            cols: usize,

            fn num_total_elements(self: Shape) usize {
                return self.rows * self.cols;
            }

            fn index_at(self: Shape, r: usize, c: usize) usize {
                return self.cols * r + c;
            }
            fn checked_index_at(self: Shape, r: usize, c: usize) usize {
                if (r < self.rows and c < self.cols) {
                    return self.index_at(r, c);
                } else {
                    const text = "(row: {}, col: {}) out of bounds shape ({}, {})";
                    std.debug.panic(text, .{ r, c, self.rows, self.cols });
                }
            }

            fn checked_row_range(self: Shape, r: usize) struct { from: usize, to: usize } {
                if (r + 1 <= self.rows) {
                    const start = r * self.cols;
                    return .{
                        .from = start,
                        .to = start + self.cols,
                    };
                } else {
                    const text = "(row: {}) out of bounds rows ({})";
                    std.debug.panic(text, .{ r, self.rows });
                }
            }
        };

        data: []T,

        shape: Shape,

        allocator: ?Allocator = null,

        fn new(allocator: ?Allocator, shape: Shape) Self {
            return Self{ .data = undefined, .shape = shape, .allocator = allocator };
        }

        pub fn init(allocator: Allocator, shape: Shape, fill_elem: T) Self {
            var self = Self.new(allocator, shape);

            self.data = allocator.alloc(T, shape.num_total_elements()) catch @panic("alloc err");
            if (fill_elem != undefined) {
                @memset(self.data, fill_elem);
            }

            return self;
        }

        /// ### Creates a new matrix with given dimensions, using the provided data buffer.
        ///
        /// Arguments:
        /// - rows - Number of rows in the matrix
        /// - cols - Number of columns in the matrix
        /// - backed - Data buffer to populate the matrix with
        ///
        /// The caller keeps ownership of the passed data buffer.
        ///
        /// The buffer size must match rows * cols.
        ///
        /// # Panics
        ///
        /// Panics if the buffer size does not match the matrix dimensions.
        pub fn new_with(shape: Shape, backed: []T) Self {
            var self = Self.new(null, shape);

            if (backed.len != shape.num_total_elements()) {
                @panic("shape mismatch");
            }
            self.data = backed;

            return self;
        }

        pub fn init_copy_from(allocator: Allocator, shape: Shape, dst: []T) Self {
            var self = Self.new(allocator, shape);

            if (dst.len != shape.num_total_elements()) {
                @panic("shape mismatch");
            }
            self.data = allocator.dupe(T, dst) catch @panic("alloc err");

            return self;
        }

        fn deinit(self: Self) void {
            const allocator = self.allocator orelse return;
            allocator.free(self.data);
        }

        fn at(self: Self, r: usize, c: usize) T {
            const index = self.shape.checked_index_at(r, c);
            return self.data[index];
        }

        fn at_mut(self: Self, r: usize, c: usize) *T {
            const index = self.shape.checked_index_at(r, c);
            return &self.data[index];
        }

        fn row_view(self: Self, r: usize) []T {
            const index = self.shape.checked_row_range(r);
            return self.data[index.from..index.to];
        }
    };
}

pub const RomuDuoJr = struct {
    x_state: u64,
    y_state: u64,

    pub fn init(seed: u64) RomuDuoJr {
        return RomuDuoJr{ .x_state = seed, .y_state = 2411094284759029579 };
    }

    pub fn random(self: *RomuDuoJr) Random {
        return Random.init(self, fill);
    }

    pub fn fill(self: *RomuDuoJr, buf: []u8) void {
        var i: usize = 0;
        const aligned_len = buf.len - (buf.len & 7);

        // Complete 8 byte segments.
        while (i < aligned_len) : (i += 8) {
            var n = self.next();
            comptime var j: usize = 0;
            inline while (j < 8) : (j += 1) {
                buf[i + j] = @as(u8, @truncate(n));
                n >>= 8;
            }
        }

        // Remaining. (cuts the stream)
        if (i != buf.len) {
            var n = self.next();
            while (i < buf.len) : (i += 1) {
                buf[i] = @as(u8, @truncate(n));
                n >>= 8;
            }
        }
    }

    pub fn next(self: *RomuDuoJr) u64 {
        const xp = self.x_state;

        self.x_state = self.y_state *% 15241094284759029579;
        self.y_state = std.math.rotl(u64, self.y_state -% xp, 27);

        return xp;
    }

    pub fn next_f64(self: *RomuDuoJr) f64 {
        const B: u64 = 64;
        const F: u64 = 53 - 1;
        const E: u64 = (1 << (B - 2)) - (1 << F);

        var bits = self.next();
        bits >>= B - F;
        bits += E;

        return @as(f64, @bitCast(bits)) - 1.0;
    }
};
