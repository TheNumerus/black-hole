use cgmath::{InnerSpace, Vector3, Zero};
use criterion::{criterion_group, criterion_main, Criterion};

use blackhole::shader::{BackgroundShader, Parameter, Shader};
use blackhole::{Ray, RayKind};
use blackhole_common::shaders::StarSkyShader;

pub fn star_sky(c: &mut Criterion) {
    let mut shader = StarSkyShader::new();
    shader.set_parameter("star_count", Parameter::Usize(42_000));

    let ray = Ray {
        location: Vector3::zero(),
        direction: Vector3::new(0.5, 0.5, 0.5).normalize(),
        steps_taken: 5,
        kind: RayKind::Primary,
    };

    c.bench_function("star_sky", |b| b.iter(|| shader.emission_at(&ray)));
}

criterion_group!(benches, star_sky);
criterion_main!(benches);
