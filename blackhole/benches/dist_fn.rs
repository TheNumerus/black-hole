use cgmath::Vector3;
use criterion::{criterion_group, criterion_main, Criterion};

use blackhole::object::shape::Cylinder;
use blackhole::object::shape::Shape;

pub fn cylinder_dist(c: &mut Criterion) {
    let cylinder = Cylinder::new();

    c.bench_function("cylinder", |b| {
        b.iter(|| cylinder.dist_fn(Vector3::new(0.0, 0.0, 0.0)))
    });
}

criterion_group!(benches, cylinder_dist);
criterion_main!(benches);
