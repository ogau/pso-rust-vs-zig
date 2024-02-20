use pso::{Params, Range, PSO};
use std::time::Instant;

fn dixonprice(x: &[f64]) -> f64 {
    let pow2 = |x| x * x;
    let mut sum = pow2(x[0] - 1.0);
    for i in 1..x.len() {
        sum += (i + 1) as f64 * pow2(2.0 * pow2(x[i]) - x[i - 1]);
    }
    sum
}

fn main() {
    let start_time = Instant::now();

    const DIMS: usize = 200;

    let n_particles = 500;

    let params = Params::<DIMS> {
        velocity_decay_factor: 0.99,
        init_speed: 0.01,

        n_particles,
        ranges: [Range::new(-10.0, 10.0); DIMS],
        ..Default::default()
    };

    let mut opt = PSO::new(params);

    let mut buf = Vec::with_capacity(n_particles);

    let total_num_generations = 20000;
    for num_gen in 0..total_num_generations {
        buf.clear();

        for solution in opt.ask() {
            let eval = dixonprice(solution);
            buf.push(eval);
        }

        let best_obj = opt.tell(&buf);
        if (num_gen + 1) % 250 == 0 {
            println!("{} {:.4}", num_gen, best_obj);
        }
    }

    println!("{:?}", start_time.elapsed());
}
