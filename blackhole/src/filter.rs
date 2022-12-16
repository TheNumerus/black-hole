use crate::lut::LookupTable;
use crate::math::blackman_harris;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;

pub trait PixelFilter: Iterator<Item = (f64, f64)> + Send + Sync {
    fn set_filter_size(&mut self, filter_size: f64);
    fn reset(&mut self);
}

///
/// Sub pixel sampler with Box window-function
///
pub struct BoxFilter {
    pub(crate) generator: Xoshiro256StarStar,
    first_sample: bool,
    filter_size: f64,
}

impl BoxFilter {
    pub fn new(filter_size: f64) -> Self {
        let generator = Xoshiro256StarStar::seed_from_u64(0);

        Self {
            generator,
            first_sample: true,
            filter_size,
        }
    }
}

impl PixelFilter for BoxFilter {
    fn set_filter_size(&mut self, filter_size: f64) {
        self.filter_size = filter_size;
    }

    fn reset(&mut self) {
        self.generator = Xoshiro256StarStar::seed_from_u64(0);
    }
}

impl Iterator for BoxFilter {
    type Item = (f64, f64);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.first_sample {
            let range = -(self.filter_size / 2.0)..(self.filter_size / 2.0);

            let x = self.generator.gen_range(range.clone());
            let y = self.generator.gen_range(range);

            Some((x + 0.5, y + 0.5))
        } else {
            self.first_sample = false;
            Some((0.5, 0.5))
        }
    }
}

pub struct BlackmanHarrisFilter {
    pub(crate) generator: Xoshiro256StarStar,
    first_sample: bool,
    filter_size: f64,
    lut: LookupTable<f64>,
}

impl BlackmanHarrisFilter {
    pub fn new(filter_size: f64) -> Self {
        let generator = Xoshiro256StarStar::seed_from_u64(0);

        let lut = Self::generate_lut();

        Self {
            generator,
            first_sample: false,
            filter_size,
            lut,
        }
    }

    fn generate_lut() -> LookupTable<f64> {
        let mut vec = Vec::new();

        let mut integral = 0.0;
        let mut last_integral = 0.0;

        for i in 0..1000 {
            let x = i as f64 / 1000.0;

            let f = blackman_harris(x, 1.0);

            integral += f * 0.001 + ((last_integral - f) / 2.0 * 0.001);

            last_integral = f;

            vec.push((x, integral));
        }

        let last = vec.last().unwrap().1;

        // normalize
        for (y, i) in vec.iter_mut() {
            *i *= 1.0 / last;

            std::mem::swap(y, i);
        }

        LookupTable::from_vec_sorted(vec)
    }
}

impl PixelFilter for BlackmanHarrisFilter {
    fn set_filter_size(&mut self, filter_size: f64) {
        self.filter_size = filter_size;
    }

    fn reset(&mut self) {
        self.generator = Xoshiro256StarStar::seed_from_u64(0);
    }
}

impl Iterator for BlackmanHarrisFilter {
    type Item = (f64, f64);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.first_sample {
            let range = (0.0)..(1.0);

            let x = self.generator.gen_range(range.clone());
            let y = self.generator.gen_range(range);

            let x = (self.lut.lookup(x) - 0.5) * self.filter_size;
            let y = (self.lut.lookup(y) - 0.5) * self.filter_size;

            Some((x + 0.5, y + 0.5))
        } else {
            self.first_sample = false;
            Some((0.5, 0.5))
        }
    }
}
