use crate::Ray;
use cgmath::Vector3;

#[derive(Copy, Clone, Debug)]
pub struct AABB {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub z_min: f64,
    pub z_max: f64,
}

impl AABB {
    pub fn new() -> Self {
        Self {
            x_min: 0.0,
            x_max: 1.0,
            y_min: 0.0,
            y_max: 1.0,
            z_min: 0.0,
            z_max: 1.0,
        }
    }

    fn min(&self) -> Vector3<f64> {
        Vector3::new(self.x_min, self.y_min, self.z_min)
    }

    fn max(&self) -> Vector3<f64> {
        Vector3::new(self.x_max, self.y_max, self.z_max)
    }

    pub fn ray_intersect(&self, ray: &Ray) -> bool {
        let (mut tmax, mut tmin) = (f64::MAX, f64::MIN);
        for a in 0..3 {
            let inv_dir = 1.0 / ray.direction[a];
            let mut t0 = (self.min()[a] - ray.location[a]) * inv_dir;
            let mut t1 = (self.max()[a] - ray.location[a]) * inv_dir;

            if inv_dir < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }

            tmin = if t0 > tmin { t0 } else { tmin };
            tmax = if t1 < tmax { t1 } else { tmax };

            if tmax <= tmin {
                return false;
            }
        }

        true
    }
}

impl Default for AABB {
    fn default() -> Self {
        Self::new()
    }
}
