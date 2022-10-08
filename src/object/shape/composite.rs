use super::Shape;
use crate::object::AABB;
use cgmath::Vector3;

pub enum Composite {
    Diff(Box<dyn Shape>, Box<dyn Shape>),
}

impl Shape for Composite {
    fn dist_fn(&self, point: Vector3<f64>) -> f64 {
        match self {
            Composite::Diff(a, b) => {
                let a = a.dist_fn(point.clone());
                let b = b.dist_fn(point);

                (a).max(-b)
            }
        }
    }

    fn bounding_box(&self) -> AABB {
        match self {
            Composite::Diff(a, b) => {
                let abb = a.bounding_box();
                let bbb = b.bounding_box();
                AABB {
                    x_min: abb.x_min.min(bbb.x_min),
                    x_max: abb.x_max.max(bbb.x_max),
                    y_min: abb.y_min.min(bbb.y_min),
                    y_max: abb.y_max.max(bbb.y_max),
                    z_min: abb.z_min.min(bbb.z_min),
                    z_max: abb.z_max.max(bbb.z_max),
                }
            }
        }
    }
}
