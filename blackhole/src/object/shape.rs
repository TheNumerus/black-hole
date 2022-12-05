use crate::object::AABB;
use crate::Ray;
use cgmath::{Array, InnerSpace, Vector3};

mod composite;
mod cylinder;
mod sphere;

pub use composite::Composite;
pub use cylinder::Cylinder;
pub use sphere::Sphere;

pub trait Shape: Send + Sync {
    fn dist_fn(&self, point: Vector3<f64>) -> f64;
    fn bounding_box(&self) -> AABB;

    fn can_ray_hit(&self, ray: &Ray) -> bool {
        let bb = self.bounding_box();

        bb.ray_intersect(ray)
    }

    fn normal(&self, position: Vector3<f64>, epsilon: f64) -> Vector3<f64> {
        let eps = 0.00001;

        let dist_x = self.dist_fn(position + Vector3::new(epsilon, 0.0, 0.0));
        let dist_y = self.dist_fn(position + Vector3::new(0.0, epsilon, 0.0));
        let dist_z = self.dist_fn(position + Vector3::new(0.0, 0.0, epsilon));

        let normal = (Vector3::new(dist_x, dist_y, dist_z)
            - Vector3::from_value(self.dist_fn(position)))
            / eps;

        normal.normalize()
    }
}
