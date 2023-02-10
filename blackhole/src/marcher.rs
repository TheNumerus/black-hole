use crate::material::MaterialResult;
use crate::math::rand_unit;
use crate::object::{Object, Shading};
use crate::scene::Scene;
use crate::{Ray, RenderMode};
use cgmath::{Array, ElementWise, InnerSpace, Vector3, Zero};

pub struct RayMarcher {
    pub mode: RenderMode,
    pub samples: usize,
    pub max_steps: usize,
    pub max_depth: usize,
}

impl RayMarcher {
    pub fn color_for_ray(&self, ray: Ray, scene: &Scene, max_step: f64, depth: usize) -> RayResult {
        if depth >= self.max_depth {
            return RayResult {
                steps: ray.steps_taken,
                color: Vector3::zero(),
            };
        }

        let mut ray = ray;
        let obj = self.march_to_object(&mut ray, scene, max_step);

        let mat_res = match obj {
            MarchResult::Object(obj) => {
                let (mat, new_ray) = self.get_color(&ray, self.mode, obj);

                match new_ray {
                    Some(new_ray) => {
                        ray = new_ray;
                    }
                    None => {
                        return RayResult {
                            steps: ray.steps_taken,
                            color: mat.emission,
                        };
                    }
                }

                mat
            }
            MarchResult::Background(_direction) => {
                // if background, end ray right away
                return RayResult {
                    steps: ray.steps_taken,
                    color: scene.background.emission_at(&ray),
                };
            }
            MarchResult::None => {
                return RayResult {
                    steps: ray.steps_taken,
                    color: Vector3::zero(),
                };
            }
        };

        let color_reflected = self.color_for_ray(ray, scene, max_step, depth + 1);

        let color = mat_res.emission + mat_res.albedo.mul_element_wise(color_reflected.color);

        RayResult {
            steps: color_reflected.steps,
            color,
        }
    }

    fn march_to_object<'r, 's>(
        &self,
        ray: &'r mut Ray,
        scene: &'s Scene,
        max_step: f64,
    ) -> MarchResult<'s> {
        let mut i = 0;
        let mut active_distortions = Vec::with_capacity(scene.distortions.len());

        loop {
            let mut dst = f64::MAX;

            active_distortions.clear();
            for distortion in &scene.distortions {
                if !distortion.can_ray_hit(ray) {
                    continue;
                }
                let dist = distortion.dist_fn(ray.location);
                if dist <= 0.0 {
                    active_distortions.push(distortion);
                }
                dst = dst.min(dist.max(0.1));
            }

            let mut obj = None;

            for object in &scene.objects {
                match &object.shading {
                    Shading::Solid(_) => {
                        if !object.shape.can_ray_hit(ray) && !active_distortions.is_empty() {
                            continue;
                        }

                        let obj_dist = object.shape.dist_fn(ray.location);
                        if obj_dist < dst {
                            dst = dst.min(obj_dist);
                            obj = Some(object);
                        }
                    }
                    Shading::Volumetric(shader) => {
                        let obj_dist = object.shape.dist_fn(ray.location);

                        if obj_dist < 0.0 {
                            dst = dst.min(0.01);
                            let r = rand_unit();
                            if (shader.density_at(ray.location) * dst) > r {
                                return MarchResult::Object(object);
                            }
                        } else if obj_dist < dst {
                            dst = dst.min(obj_dist.max(0.002));
                        }
                    }
                }
            }

            if let Some(obj) = obj {
                if dst < 0.00001 {
                    return MarchResult::Object(obj);
                }
            }

            for distortion in &active_distortions {
                let strength = distortion.strength(ray.location);

                if strength > 9.0 {
                    return MarchResult::None;
                }

                let force = (distortion.shape.center() - ray.location).normalize() * dst * strength;

                let new_dir = (ray.direction + force).normalize();

                if ray.direction.dot(new_dir) < -0.0 {
                    return MarchResult::None;
                }
                ray.direction = new_dir;
            }

            if dst > max_step {
                return MarchResult::Background(ray.direction);
            }

            if i >= self.max_steps {
                return MarchResult::None;
            }
            i += 1;

            ray.advance(dst);
        }
    }

    fn get_color(
        &self,
        ray: &Ray,
        render_mode: RenderMode,
        object: &Object,
    ) -> (MaterialResult, Option<Ray>) {
        let (mat, new_ray) = object.shade(ray);

        match render_mode {
            RenderMode::Shaded => (mat, new_ray),
            RenderMode::Normal => {
                let eps = 0.00001;
                let normal =
                    object.shape.normal(ray.location, eps) * 0.5 + Vector3::from_value(0.5);

                (
                    MaterialResult {
                        emission: normal,
                        albedo: Vector3::zero(),
                    },
                    new_ray,
                )
            }
            RenderMode::Samples => (
                MaterialResult {
                    emission: Vector3::zero(),
                    albedo: Vector3::zero(),
                },
                new_ray,
            ),
        }
    }
}

impl Default for RayMarcher {
    fn default() -> Self {
        Self {
            mode: RenderMode::Shaded,
            samples: 128,
            max_steps: 2 << 16,
            max_depth: 16,
        }
    }
}

pub struct RayResult {
    pub steps: usize,
    pub color: Vector3<f64>,
}

enum MarchResult<'a> {
    Object(&'a Object),
    Background(Vector3<f64>),
    None,
}
