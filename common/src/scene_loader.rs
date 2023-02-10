use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::sync::Arc;

use blackhole::scene::Scene;
use blackhole::shader::{BackgroundShader, Parameter, Shader, SolidShader, VolumetricShader};

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

        let mut shaders_solid: HashMap<String, Arc<dyn SolidShader>> = HashMap::new();
        let mut shaders_volumetric: HashMap<String, Arc<dyn VolumetricShader>> = HashMap::new();
        let mut shaders_background: HashMap<String, Arc<dyn BackgroundShader>> = HashMap::new();

        let mut shader_types: HashMap<String, ShaderType> = HashMap::new();

        for (name, shader) in &json.shaders {
            let params = shader.parameters.as_ref();

            match shader.kind.as_str() {
                "background" => {
                    let shader = build_background_shader(shader.class.as_str(), params)?;

                    shaders_background.insert(name.clone(), shader);
                    shader_types.insert(name.clone(), ShaderType::Background);
                }
                "volumetric" => {
                    let shader = build_volumetric_shader(shader.class.as_str(), params)?;

                    shaders_volumetric.insert(name.clone(), shader);
                    shader_types.insert(name.clone(), ShaderType::Volumetric);
                }
                "solid" => {
                    let shader = build_solid_shader(shader.class.as_str(), params)?;

                    shaders_solid.insert(name.clone(), shader);
                    shader_types.insert(name.clone(), ShaderType::Solid);
                }
                _ => return Err(LoaderError::Other("unknown shader category".into())),
            }
        }

        let bg_name = json.background;
        let bg = shaders_background
            .get(&bg_name)
            .ok_or(LoaderError::IndexError(bg_name, "background shaders"))?;

        let mut scene = Scene::new(Arc::clone(bg));

        for stub in &json.objects {
            let st = match shader_types.get(&stub.shader) {
                Some(st) => st,
                None => return Err(LoaderError::IndexError(stub.shader.clone(), "shaders")),
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

fn build_background_shader(
    name: &str,
    params: Option<&HashMap<String, ParameterValue>>,
) -> Result<Arc<dyn BackgroundShader>, LoaderError> {
    match name {
        "StarSkyShader" => Ok(Arc::new(build_shader::<StarSkyShader>(params))),
        "SolidColorBackgroundShader" => {
            Ok(Arc::new(build_shader::<SolidColorBackgroundShader>(params)))
        }
        "DebugBackgroundShader" => Ok(Arc::new(build_shader::<DebugBackgroundShader>(params))),
        _ => Err(LoaderError::Other("unknown background shader".into())),
    }
}

fn build_volumetric_shader(
    name: &str,
    params: Option<&HashMap<String, ParameterValue>>,
) -> Result<Arc<dyn VolumetricShader>, LoaderError> {
    match name {
        "BlackHoleEmitterShader" => Ok(Arc::new(build_shader::<BlackHoleEmitterShader>(params))),
        "BlackHoleScatterShader" => Ok(Arc::new(build_shader::<BlackHoleScatterShader>(params))),
        "VolumeEmitterShader" => Ok(Arc::new(build_shader::<VolumeEmitterShader>(params))),
        "SolidColorVolumeShader" => Ok(Arc::new(build_shader::<SolidColorVolumeShader>(params))),
        "SolidColorVolumeAbsorbShader" => Ok(Arc::new(
            build_shader::<SolidColorVolumeAbsorbShader>(params),
        )),
        "SolidColorVolumeScatterShader" => Ok(Arc::new(build_shader::<
            SolidColorVolumeScatterShader,
        >(params))),
        "DebugNoiseVolumeShader" => Ok(Arc::new(build_shader::<DebugNoiseVolumeShader>(params))),
        _ => Err(LoaderError::Other("unknown volumetric shader".into())),
    }
}

fn build_solid_shader(
    name: &str,
    params: Option<&HashMap<String, ParameterValue>>,
) -> Result<Arc<dyn SolidShader>, LoaderError> {
    match name {
        "BasicSolidShader" => Ok(Arc::new(build_shader::<BasicSolidShader>(params))),
        _ => Err(LoaderError::Other("unknown solid shader".into())),
    }
}

fn build_shader<T>(parameters: Option<&HashMap<String, ParameterValue>>) -> T
where
    T: Shader + Default,
{
    let mut shader = T::default();

    if let Some(params) = parameters {
        for (name, value) in params {
            let value = match value {
                ParameterValue::Vec3(v) => Parameter::Vec3(Vector3::from(*v)),
                ParameterValue::U64(u) => Parameter::Usize(*u as usize),
                ParameterValue::Float(f) => Parameter::Float(*f),
            };

            shader.set_parameter(name, value);
        }
    }

    shader
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
    IndexError(String, &'static str),
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
    shader: String,
    shape: Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ShaderStub {
    class: String,
    kind: String,
    parameters: Option<HashMap<String, ParameterValue>>,
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
    background: String,
    shaders: BTreeMap<String, ShaderStub>,
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
