use crate::object::Object;
use crate::Distortion;
use cgmath::Vector3;

pub struct Scene {
    pub objects: Vec<Object>,
    pub distortions: Vec<Distortion>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            distortions: Vec::new(),
        }
    }

    pub fn push(mut self, item: Object) -> Self {
        self.objects.push(item);

        self
    }

    pub fn max_possible_step(&self, origin: Vector3<f64>) -> f64 {
        let [mut min_x, mut max_x, mut min_y, mut max_y, mut min_z, mut max_z] =
            [origin.x, origin.x, origin.y, origin.y, origin.z, origin.z];

        for object in &self.objects {
            let bb = object.shape.bounding_box();
            min_x = min_x.min(bb.x_min);
            max_x = max_x.max(bb.x_max);
            min_y = min_y.min(bb.y_min);
            max_y = max_y.max(bb.y_max);
            min_z = min_z.min(bb.z_min);
            max_z = max_z.max(bb.z_max);
        }
        let delta_x = max_x - min_x;
        let delta_y = max_y - min_y;
        let delta_z = max_z - min_y;

        let delta_xy = (delta_x * delta_x + delta_y * delta_y).sqrt();
        (delta_xy * delta_xy + delta_z * delta_z).sqrt()
    }
}
