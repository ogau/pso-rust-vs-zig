pub use romu_duo_jr::*;
mod romu_duo_jr {
    use rand_core::{le::read_u64_into, RngCore, SeedableRng};

    const NUM_FIELDS: usize = 2;
    type T = u64;
    pub struct RomuDuoJr {
        x_state: T,
        y_state: T,
    }

    impl RngCore for RomuDuoJr {
        #[inline]
        fn next_u32(&mut self) -> u32 {
            (self.next_u64() >> 32) as u32
        }

        #[inline]
        fn next_u64(&mut self) -> u64 {
            let xp = self.x_state;

            self.x_state = self.y_state.wrapping_mul(15241094284759029579);
            self.y_state = self.y_state.wrapping_sub(xp).rotate_left(27);

            xp
        }

        #[inline]
        fn fill_bytes(&mut self, buf: &mut [u8]) {
            rand_core::impls::fill_bytes_via_next(self, buf);
        }

        #[inline]
        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }

    impl SeedableRng for RomuDuoJr {
        type Seed = [u8; std::mem::size_of::<T>() * NUM_FIELDS];

        #[inline]
        fn from_seed(seed: Self::Seed) -> Self {
            let mut dst = [0; NUM_FIELDS];
            read_u64_into(&seed, &mut dst);
            let [x, y] = dst;
            Self {
                x_state: x,
                y_state: y,
            }
        }
    }
}
pub use romu_trio::*;
mod romu_trio {
    use rand_core::{RngCore, SeedableRng};

    const NUM_FIELDS: usize = 3;
    type T = u64;
    pub struct RomuTrio {
        x_state: T,
        y_state: T,
        z_state: T,
    }

    impl RngCore for RomuTrio {
        #[inline]
        fn next_u32(&mut self) -> u32 {
            (self.next_u64() >> 32) as u32
        }

        #[inline]
        fn next_u64(&mut self) -> u64 {
            let xp = self.x_state;
            let yp = self.y_state;
            let zp = self.z_state;

            self.x_state = zp.wrapping_mul(15241094284759029579);
            self.y_state = yp.wrapping_sub(xp);

            self.y_state = self.y_state.rotate_left(12);
            self.z_state = zp.wrapping_sub(yp);
            self.z_state = self.z_state.rotate_left(44);

            xp
        }

        #[inline]
        fn fill_bytes(&mut self, buf: &mut [u8]) {
            rand_core::impls::fill_bytes_via_next(self, buf);
        }

        #[inline]
        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }

    impl SeedableRng for RomuTrio {
        type Seed = [u8; std::mem::size_of::<T>() * NUM_FIELDS];

        #[inline]
        fn from_seed(seed: Self::Seed) -> Self {
            let [x, y, z] = unsafe { std::mem::transmute::<Self::Seed, [T; NUM_FIELDS]>(seed) };

            Self {
                x_state: x,
                y_state: y,
                z_state: z,
            }
        }
    }
}
