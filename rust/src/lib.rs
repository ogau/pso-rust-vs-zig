use dense::Dense;
use rand::{Rng, RngCore, SeedableRng};
use rand_distr::StandardNormal;

mod random;

#[derive(Clone, Copy)]
pub struct Range {
    pub min: f64,
    pub max: f64,
}

impl Range {
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    fn randomize<R: RngCore>(self, rng: &mut R) -> f64 {
        itof(rng.next_u64()) * (self.max - self.min) + self.min
    }
}

pub struct Params<const DIMS: usize> {
    pub n_particles: usize,
    pub ranges: [Range; DIMS],
    pub cognitive_factor: f64,
    pub social_factor: f64,
    pub velocity_decay_factor: f64,
    pub init_speed: f64,
}

impl<const DIMS: usize> Default for Params<DIMS> {
    fn default() -> Self {
        Self {
            n_particles: 20,
            ranges: [Range::new(0.0, 1.0); DIMS],
            cognitive_factor: 0.1,
            social_factor: 0.1,
            velocity_decay_factor: 0.8,
            init_speed: 0.1,
        }
    }
}

type DefaultRng = random::RomuDuoJr;

pub struct PSO<const DIMS: usize> {
    params: Params<DIMS>,
    positions: Dense<f64>,
    velocities: Dense<f64>,
    best_positions: Dense<f64>,
    best_objectives: Box<[f64]>,
    rng: DefaultRng,
}

impl<const DIMS: usize> PSO<DIMS> {
    pub fn new(params: Params<DIMS>) -> Self {
        let Params {
            n_particles,
            ranges,
            init_speed,
            ..
        } = params;

        let mut rng = DefaultRng::from_entropy();

        let mut velocities = Dense::new(n_particles, DIMS);
        let mut positions = Dense::new(n_particles, DIMS);

        let best_objectives = (0..n_particles).map(|_| f64::INFINITY).collect();

        for i in 0..n_particles {
            let row_mut = positions.row_view_mut(i);
            for j in 0..DIMS {
                row_mut[j] = ranges[j].randomize(&mut rng);
            }
        }
        let best_positions = positions.clone();

        let mut rand_normal = || -> f64 { rng.sample(StandardNormal) };
        velocities.data_mut().iter_mut().for_each(|v| {
            *v = rand_normal() * init_speed;
        });

        Self {
            params,
            positions,
            velocities,
            best_positions,
            best_objectives,
            rng,
        }
    }

    pub fn ask(&self) -> dense::IterRows<'_, f64> {
        self.positions.iter_rows()
    }

    pub fn tell(&mut self, objectives: &[f64]) -> f64 {
        let obj_it = objectives.iter();
        let best_obj_it_mut = self.best_objectives.iter_mut();

        let zipped = obj_it.zip(best_obj_it_mut).enumerate();
        let filtered = zipped.filter(|(_, (&obj, &mut best_obj))| obj < best_obj);

        for (i, (&obj, best_obj)) in filtered {
            *best_obj = obj;

            let best_position = self.best_positions.row_view_mut(i);
            let position = self.positions.row_view(i);
            best_position.copy_from_slice(position);
        }

        let global_best_index = argmin(&self.best_objectives);

        let global_best_positions = self.best_positions.row_view(global_best_index);
        let global_best_objective = self.best_objectives[global_best_index];

        let Params {
            n_particles,
            cognitive_factor,
            social_factor,
            velocity_decay_factor,
            ..
        } = self.params;

        let [mut rand_factor_1, mut rand_factor_2]: [f64; 2];

        for i in 0..n_particles {
            let positions = self.positions.row_view_mut(i);
            let velocities = self.velocities.row_view_mut(i);
            let best_positions = self.best_positions.row_view(i);
            for d in 0..DIMS {
                let updated_velocity = {
                    [rand_factor_1, rand_factor_2] = self.rng.gen::<[u64; 2]>().map(itof);

                    let cognitive_component =
                        cognitive_factor * rand_factor_1 * (best_positions[d] - positions[d]);
                    let social_component =
                        social_factor * rand_factor_2 * (global_best_positions[d] - positions[d]);

                    velocity_decay_factor * velocities[d] + cognitive_component + social_component
                };

                velocities[d] = updated_velocity;
                positions[d] += updated_velocity;
            }
        }

        global_best_objective
    }
}

fn argmin(slice: &[f64]) -> usize {
    let (ind, _) = slice
        .iter()
        .enumerate()
        .reduce(|a, b| if a.1 < b.1 { a } else { b })
        .unwrap();

    ind
}

#[inline]
pub fn itof(mut bits: u64) -> f64 {
    const B: u32 = 64;
    const F: u32 = core::f64::MANTISSA_DIGITS - 1;
    const E: u64 = (1 << (B - 2)) - (1 << F);

    bits >>= B - F;
    bits += E;

    f64::from_bits(bits) - 1.0
}

#[inline(always)]
pub fn itof2<R: RngCore>(r: &mut R) -> f64 {
    let rand = r.next_u64();
    let mut rand_lz: u64 = rand.leading_zeros() as u64;
    if rand_lz >= 12 {
        rand_lz = 12;
        loop {
            // It is astronomically unlikely for this loop to execute more than once.
            let addl_rand_lz = r.next_u64().leading_zeros() as u64;
            rand_lz += addl_rand_lz;
            if addl_rand_lz != 64 {
                break;
            }
            if rand_lz >= 1022 {
                rand_lz = 1022;
                break;
            }
        }
    }
    let mantissa = rand & 0xFFFFFFFFFFFFF;
    let exponent = (1022 - rand_lz) << 52;
    f64::from_bits(exponent | mantissa)
}
