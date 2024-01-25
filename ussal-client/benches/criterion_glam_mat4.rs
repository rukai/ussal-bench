// Glam mat4 benchmarks taken from https://github.com/bitshifter/glam-rs/blob/main/benches/mat4.rs

mod support;

use criterion::{criterion_group, criterion_main, Criterion};
use std::ops::Mul;
use support::*;

bench_binop!(
    mat4_mul_vec4,
    "mat4 mul vec4",
    op => mul,
    from1 => random_srt_mat4,
    from2 => random_vec4
);

bench_binop!(
    mat4_transform_point3,
    "mat4 transform point3",
    op => transform_point3,
    from1 => random_srt_mat4,
    from2 => random_vec3
);

bench_binop!(
    mat4_transform_vector3,
    "mat4 transform vector3",
    op => transform_vector3,
    from1 => random_srt_mat4,
    from2 => random_vec3
);

bench_binop!(
    mat4_transform_point3a,
    "mat4 transform point3a",
    op => transform_point3a,
    from1 => random_srt_mat4,
    from2 => random_vec3a
);

bench_binop!(
    mat4_transform_vector3a,
    "mat4 transform vector3a",
    op => transform_vector3a,
    from1 => random_srt_mat4,
    from2 => random_vec3a
);

bench_binop!(
    mat4_mul_mat4,
    "mat4 mul mat4",
    op => mul,
    from1 => random_srt_mat4,
    from2 => random_srt_mat4
);

// reasoning for external input/output https://github.com/bitshifter/glam-rs/commit/f0f760e32decb131f067c98921111ce9f9793101
#[macro_export]
macro_rules! bench_binop {
    ($name: ident, $desc: expr, op => $binop:ident, from1 => $from1:expr, from2 => $from2:expr) => {
        pub(crate) fn $name(c: &mut Criterion) {
            const SIZE: usize = 1 << 13;
            let mut rng = support::PCG32::default();
            let inputs1 =
                criterion::black_box((0..SIZE).map(|_| $from1(&mut rng)).collect::<Vec<_>>());
            let inputs2 =
                criterion::black_box((0..SIZE).map(|_| $from2(&mut rng)).collect::<Vec<_>>());
            // pre-fill output vector with some random value
            let mut outputs = vec![$from1(&mut rng).$binop($from2(&mut rng)); SIZE];
            let mut i = 0;
            c.bench_function($desc, |b| {
                b.iter(|| {
                    i = (i + 1) & (SIZE - 1);
                    unsafe {
                        *outputs.get_unchecked_mut(i) =
                            inputs1.get_unchecked(i).$binop(*inputs2.get_unchecked(i));
                    }
                })
            });
            criterion::black_box(outputs);
        }
    };
}

criterion_group!(
    benches,
    mat4_mul_mat4,
    mat4_mul_vec4,
    mat4_transform_point3,
    mat4_transform_point3a,
    mat4_transform_vector3,
    mat4_transform_vector3a,
);

criterion_main!(benches);
