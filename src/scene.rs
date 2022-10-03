use crate::object::Renderable;
use crate::Distortion;
use cgmath::Vector3;

pub struct Scene {
    pub objects: Vec<Box<dyn Renderable>>,
    pub distortions: Vec<Distortion>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            distortions: Vec::new(),
        }
    }

    pub fn push(mut self, item: Box<dyn Renderable>) -> Self {
        self.objects.push(item);

        self
    }

    pub fn max_possible_step(&self, origin: Vector3<f64>) -> f64 {
        let [mut min_x, mut max_x, mut min_y, mut max_y, mut min_z, mut max_z] =
            [origin.x, origin.x, origin.y, origin.y, origin.z, origin.z];

        for object in &self.objects {
            if let Some(bb) = object.bounding_box() {
                min_x = min_x.min(bb[0]);
                max_x = max_x.max(bb[1]);
                min_y = min_y.min(bb[2]);
                max_y = max_y.max(bb[3]);
                min_z = min_z.min(bb[4]);
                max_z = max_z.max(bb[5]);
            }
        }
        let delta_x = max_x - min_x;
        let delta_y = max_y - min_y;
        let delta_z = max_z - min_y;

        let delta_xy = (delta_x * delta_x + delta_y * delta_y).sqrt();
        (delta_xy * delta_xy + delta_z * delta_z).sqrt()
    }
}
