use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::sync::Arc;

use blackhole::scene::Scene;
use blackhole::shader::{BackgroundShader, SolidShader, VolumetricShader};

use cgmath::Vector3;

use serde::{Deserialize, Serialize};

use blackhole::camera::Camera;
use serde_json::{Map, Value};

use blackhole::object::shape::{Composite, Cube, Cylinder, Shape, Sphere};
use blackhole::object::{Distortion, Object};

use crate::shaders::*;

macro_rules! extract_vec3 {
    ($stub:ident, $shape:ident, $method:path, $name:literal) => {
        if let Some(item) = $stub.get($name) {
            let item = item.as_array().ok_or_else(|| {
                let msg = format!("wrong {} type", $name);
                LoaderError::Other(msg)
            })?;

            let vec3 = arr_to_vec3(item)?;

            $method(&mut $shape, vec3);
        }
    };
}

macro_rules! extract_float {
    ($stub:ident, $shape:ident, $method:path, $name:literal) => {
        if let Some(item) = $stub.get($name) {
            let float = item.as_f64().ok_or_else(|| {
                let msg = format!("wrong {} type", $name);
                LoaderError::Other(msg)
            })?;

            $method(&mut $shape, float);
        }
    };
}

pub struct SceneLoader {}

impl SceneLoader {
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Scene, LoaderError> {
        let scene_str = std::fs::read_to_string(path).map_err(LoaderError::InputError)?;

        let json: SceneFile = json5::from_str(&scene_str).map_err(LoaderError::FormatError)?;

        let mut shaders_solid: HashMap<usize, Arc<dyn SolidShader>> = HashMap::new();
        let mut shaders_volumetric: HashMap<usize, Arc<dyn VolumetricShader>> = HashMap::new();
        let mut shaders_background: HashMap<usize, Arc<dyn BackgroundShader>> = HashMap::new();

        let mut shader_types: HashMap<usize, ShaderType> = HashMap::new();

        for shader in &json.shaders {
            let params = shader
                .parameters
                .as_ref()
                .ok_or(LoaderError::Other("missing parameters for shader".into()));

            match shader.kind.as_str() {
                "background" => {
                    match shader.class.as_str() {
                        "StarSkyShader" => {
                            let (stars, color) = match params?.as_slice() {
                                [ParameterValue::U64(s), ParameterValue::Vec3(a)] => {
                                    (*s, Vector3::from(*a))
                                }
                                _ => {
                                    return Err(LoaderError::Other(
                                        "invalid parameters for StarSkyShader".into(),
                                    ))
                                }
                            };

                            let specific_shader = StarSkyShader::new(stars as usize, color);

                            shaders_background.insert(shader.id, Arc::new(specific_shader));
                        }
                        "SolidColorBackgroundShader" => {
                            let color = match params?.as_slice() {
                                [ParameterValue::Vec3(a)] => Vector3::from(*a),
                                _ => {
                                    return Err(LoaderError::Other(
                                        "invalid parameters for SolidColorBackgroundShader".into(),
                                    ))
                                }
                            };

                            shaders_background.insert(
                                shader.id,
                                Arc::new(SolidColorBackgroundShader::new(color)),
                            );
                        }
                        "DebugBackgroundShader" => {
                            shaders_background
                                .insert(shader.id, Arc::new(DebugBackgroundShader {}));
                        }
                        _ => return Err(LoaderError::Other("unknown background shader".into())),
                    }
                    shader_types.insert(shader.id, ShaderType::Background);
                }
                "volumetric" => {
                    match shader.class.as_str() {
                        "BlackHoleEmitterShader" => {
                            shaders_volumetric
                                .insert(shader.id, Arc::new(BlackHoleEmitterShader::new()));
                        }
                        "BlackHoleScatterShader" => {
                            shaders_volumetric
                                .insert(shader.id, Arc::new(BlackHoleScatterShader::new()));
                        }
                        "VolumeEmitterShader" => {
                            let (temp, density, strength) = match params?.as_slice() {
                                [ParameterValue::Float(t), ParameterValue::Float(d), ParameterValue::Float(s)] => {
                                    (*t, *d, *s)
                                }
                                _ => {
                                    return Err(LoaderError::Other(
                                        "invalid parameters for VolumeEmitterShader".into(),
                                    ))
                                }
                            };

                            shaders_volumetric.insert(
                                shader.id,
                                Arc::new(VolumeEmitterShader::new(temp, density, strength)),
                            );
                        }
                        "SolidColorVolumeShader" => {
                            let (albedo, density) = match params?.as_slice() {
                                [ParameterValue::Vec3(a), ParameterValue::Float(d)] => {
                                    (Vector3::from(*a), *d)
                                }
                                _ => {
                                    return Err(LoaderError::Other(
                                        "invalid parameters for SolidColorVolumeShader".into(),
                                    ))
                                }
                            };

                            shaders_volumetric.insert(
                                shader.id,
                                Arc::new(SolidColorVolumeShader::new(albedo, density)),
                            );
                        }
                        "DebugNoiseVolumeShader" => {
                            shaders_volumetric
                                .insert(shader.id, Arc::new(DebugNoiseVolumeShader::new()));
                        }
                        _ => return Err(LoaderError::Other("unknown volumetric shader".into())),
                    }
                    shader_types.insert(shader.id, ShaderType::Volumetric);
                }
                "solid" => {
                    match shader.class.as_str() {
                        "BasicSolidShader" => {
                            let (albedo, emission, metallic) = match params?.as_slice() {
                                [ParameterValue::Vec3(a), ParameterValue::Vec3(b), ParameterValue::Float(c)] => {
                                    (Vector3::from(*a), Vector3::from(*b), *c)
                                }
                                _ => {
                                    return Err(LoaderError::Other(
                                        "invalid parameters for BasicSolidShader".into(),
                                    ))
                                }
                            };

                            shaders_solid.insert(
                                shader.id,
                                Arc::new(BasicSolidShader::new(albedo, emission, metallic)),
                            );
                        }
                        _ => return Err(LoaderError::Other("unknown solid shader".into())),
                    }
                    shader_types.insert(shader.id, ShaderType::Solid);
                }
                _ => return Err(LoaderError::Other("unknown shader category".into())),
            }
        }

        let bg_index = json.background;
        let bg = shaders_background
            .get(&bg_index)
            .ok_or(LoaderError::IndexError(bg_index, "background shaders"))?;

        let mut scene = Scene::new(Arc::clone(bg));

        for stub in &json.objects {
            let st = match shader_types.get(&stub.shader) {
                Some(st) => st,
                None => return Err(LoaderError::IndexError(stub.shader, "shaders")),
            };

            let shape = build_shape(&stub.shape)?;

            let object = match st {
                ShaderType::Solid => {
                    let shader = shaders_solid.get(&stub.shader).unwrap().clone();

                    Object::solid(shape, shader)
                }
                ShaderType::Volumetric => {
                    let shader = shaders_volumetric.get(&stub.shader).unwrap().clone();

                    Object::volumetric(shape, shader)
                }
                _ => return Err(LoaderError::Other("invalid shader type".into())),
            };

            scene = scene.push(object);
        }

        scene.distortions = load_distortions(&json.distortions);
        scene.camera = load_camera(&json.camera);

        Ok(scene)
    }
}

fn build_shape(value: &Map<String, Value>) -> Result<Arc<dyn Shape>, LoaderError> {
    if value.len() != 1 {
        return Err(LoaderError::Other("invalid shape format".into()));
    }

    let (name, stub) = value.iter().next().unwrap();

    let obj = match name.as_str() {
        "composite" => {
            let op = match stub.get("op") {
                Some(op) => op
                    .as_str()
                    .ok_or(LoaderError::Other("invalid type for composite op".into()))?,
                None => return Err(LoaderError::KeyError("op")),
            };

            let a = build_shape(
                stub.get("a")
                    .ok_or(LoaderError::KeyError("a"))?
                    .as_object()
                    .ok_or(LoaderError::Other("invalid type".into()))?,
            )?;
            let b = build_shape(
                stub.get("b")
                    .ok_or(LoaderError::KeyError("b"))?
                    .as_object()
                    .ok_or(LoaderError::Other("invalid type".into()))?,
            )?;

            let composite = match op {
                "diff" => Composite::diff(a, b),
                "intersect" => Composite::intersect(a, b),
                "union" => Composite::union(a, b),
                _ => return Err(LoaderError::Other("invalid op".into())),
            };

            Arc::new(composite) as Arc<dyn Shape>
        }
        "sphere" => {
            let mut sphere = Sphere::new();

            extract_vec3!(stub, sphere, Sphere::set_center, "center");
            extract_float!(stub, sphere, Sphere::set_radius, "radius");

            Arc::new(sphere) as Arc<dyn Shape>
        }
        "cylinder" => {
            let mut cylinder = Cylinder::new();

            extract_vec3!(stub, cylinder, Cylinder::set_center, "center");
            extract_float!(stub, cylinder, Cylinder::set_radius, "radius");
            extract_float!(stub, cylinder, Cylinder::set_height, "height");

            Arc::new(cylinder) as Arc<dyn Shape>
        }
        "cube" => {
            let mut cube = Cube::new();

            extract_vec3!(stub, cube, Cube::set_scales, "scales");
            extract_vec3!(stub, cube, Cube::set_center, "center");

            Arc::new(cube) as Arc<dyn Shape>
        }
        _ => return Err(LoaderError::Other("invalid shape".into())),
    };

    Ok(obj)
}

fn arr_to_vec3(arr: &Vec<Value>) -> Result<Vector3<f64>, LoaderError> {
    if arr.len() != 3 {
        return Err(LoaderError::Other("invalid array length for vec3".into()));
    }

    let mut values = [0.0; 3];

    for (i, v) in arr.iter().enumerate() {
        match v.as_f64() {
            Some(f) => values[i] = f,
            None => return Err(LoaderError::Other("invalid value type for vec3".into())),
        }
    }

    Ok(Vector3::from(values))
}

fn load_distortions(stubs: &[DistortionStub]) -> Vec<Distortion> {
    stubs
        .iter()
        .map(|stub| {
            let mut distortion = Distortion::new();
            if let Some(str) = stub.strength {
                distortion.strength = str;
            }

            if let Some(r) = stub.radius {
                distortion.shape.set_radius(r);
            }

            if let Some(center) = &stub.center {
                let vec3 = Vector3::from(*center);

                distortion.shape.set_center(vec3);
            }

            distortion
        })
        .collect()
}

fn load_camera(stub: &CameraStub) -> Camera {
    let mut cam = Camera::new();

    if let Some(loc) = stub.location {
        cam.location = Vector3::from(loc);
    }

    if let Some(fw) = stub.rotation {
        cam.set_rotation(Vector3::from(fw));
    }

    cam.hor_fov = stub.hor_fov;

    cam
}

#[derive(Debug)]
pub enum LoaderError {
    InputError(std::io::Error),
    FormatError(json5::Error),
    IndexError(usize, &'static str),
    KeyError(&'static str),
    Other(String),
}

impl Display for LoaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InputError(e) => f.write_fmt(format_args!("{e}")),
            Self::FormatError(e) => f.write_fmt(format_args!("{e}")),
            Self::IndexError(index, kind) => {
                f.write_fmt(format_args!("no index {index} found in {kind}"))
            }
            Self::KeyError(key) => f.write_fmt(format_args!("no key '{key}' found")),
            Self::Other(e) => f.write_fmt(format_args!("{e}")),
        }
    }
}

impl Error for LoaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InputError(e) => Some(e),
            Self::FormatError(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ObjectStub {
    shader: usize,
    shape: Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ShaderStub {
    class: String,
    id: usize,
    kind: String,
    parameters: Option<Vec<ParameterValue>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DistortionStub {
    center: Option<[f64; 3]>,
    strength: Option<f64>,
    radius: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CameraStub {
    location: Option<[f64; 3]>,
    rotation: Option<[f64; 3]>,
    hor_fov: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SceneFile {
    background: usize,
    shaders: Vec<ShaderStub>,
    objects: Vec<ObjectStub>,
    distortions: Vec<DistortionStub>,
    camera: CameraStub,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
#[serde(untagged)]
enum ParameterValue {
    Vec3([f64; 3]),
    U64(u64),
    Float(f64),
}

enum ShaderType {
    Solid,
    Volumetric,
    Background,
}
