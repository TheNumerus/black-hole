use crate::math::Lerpable;

pub struct LookupTable<T: Lerpable> {
    data: Vec<(f64, T)>,
}

impl<T: Lerpable> LookupTable<T> {
    pub fn from_vec_sorted(data: Vec<(f64, T)>) -> Self {
        if data.len() <= 1 {
            panic!("LUT needs at least two items");
        }

        Self { data }
    }

    pub fn from_vec(mut data: Vec<(f64, T)>) -> Self {
        if data.len() <= 1 {
            panic!("LUT needs at least two items");
        }

        data.sort_by(|(a, _), (b, _)| a.total_cmp(&b));

        Self { data }
    }

    pub fn lookup(&self, value: f64) -> T {
        let i = match &self.data.binary_search_by(|(k, _)| k.total_cmp(&value)) {
            Ok(v) => return self.data[*v].1,
            Err(v) => *v,
        };

        let prev = (i.max(1) - 1).min(self.data.len() - 2);
        let next = i.max(1).min(self.data.len() - 1);

        let prev = &self.data[prev];
        let next = &self.data[next];

        let mut factor = (value - prev.0) / (next.0 - prev.0);

        if factor.is_infinite() {
            factor = factor.is_sign_positive() as u8 as f64;
        }

        prev.1.lerp(&next.1, factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lut_test() {
        let lut = LookupTable::from_vec(vec![(0.0, 0.0), (1.0, 2.0), (2.0, 4.0)]);

        assert_eq!(lut.lookup(-1.0), -2.0);
        assert_eq!(lut.lookup(0.0), 0.0);
        assert_eq!(lut.lookup(0.5), 1.0);
        assert_eq!(lut.lookup(1.0), 2.0);
        assert_eq!(lut.lookup(1.5), 3.0);
        assert_eq!(lut.lookup(2.0), 4.0);
        assert_eq!(lut.lookup(3.0), 6.0);
    }
}
