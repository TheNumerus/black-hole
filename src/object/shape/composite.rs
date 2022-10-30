use super::Shape;
use crate::object::AABB;
use cgmath::Vector3;

pub struct Composite {
    a: Box<dyn Shape>,
    b: Box<dyn Shape>,
    op: BooleanOp,
    bounding_box: AABB,
}

pub enum BooleanOp {
    Difference,
    #[allow(dead_code)]
    Intersection,
    #[allow(dead_code)]
    Union,
}

impl Composite {
    pub fn diff(a: Box<dyn Shape>, b: Box<dyn Shape>) -> Self {
        let mut composite = Self {
            a,
            b,
            op: BooleanOp::Difference,
            bounding_box: AABB::new(),
        };
        composite.compute_bb();
        composite
    }

    fn compute_bb(&mut self) {
        let abb = self.a.bounding_box();
        let bbb = self.b.bounding_box();

        self.bounding_box = match self.op {
            BooleanOp::Intersection | BooleanOp::Union => AABB {
                x_min: abb.x_min.min(bbb.x_min),
                x_max: abb.x_max.max(bbb.x_max),
                y_min: abb.y_min.min(bbb.y_min),
                y_max: abb.y_max.max(bbb.y_max),
                z_min: abb.z_min.min(bbb.z_min),
                z_max: abb.z_max.max(bbb.z_max),
            },
            BooleanOp::Difference => abb,
        }
    }
}

impl Shape for Composite {
    fn dist_fn(&self, point: Vector3<f64>) -> f64 {
        let a = self.a.dist_fn(point.clone());
        let b = self.b.dist_fn(point);

        match self.op {
            BooleanOp::Difference => (a).max(-b),
            BooleanOp::Intersection => a.max(b),
            BooleanOp::Union => a.min(b),
        }
    }

    fn bounding_box(&self) -> AABB {
        self.bounding_box
    }
}
