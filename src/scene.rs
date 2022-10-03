use crate::object::Renderable;

pub struct Scene {
    pub objects: Vec<Box<dyn Renderable>>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn push(mut self, item: Box<dyn Renderable>) -> Self {
        self.objects.push(item);

        self
    }
}
