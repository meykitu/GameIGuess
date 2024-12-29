use noise::{Perlin, NoiseFn};
use rayon::prelude::*;

pub fn generate_scalar_field(grid_size: usize, position: Vec<i32>) -> Vec<Vec<Vec<f32>>> {
    let noise = Perlin::new(1024);

    let global_scale = 1.2;
    let scale = 0.01 * global_scale;
    let falloff = 0.01 * global_scale;

    let field: Vec<Vec<Vec<f32>>> = (0..grid_size)
        .into_par_iter()
        .map(|x| {
            (0..grid_size)
                .map(|y| {
                    (0..grid_size)
                        .map(|z| {
                            let noise_value = noise.get([
                                (x as f64 + position[0] as f64) * scale,
                                (y as f64 + position[1] as f64) * scale,
                                (z as f64 + position[2] as f64) * scale,
                            ]);

                            let final_value = noise_value - ((y as f64 - 150.0 * global_scale) * falloff);
                            final_value as f32
                        })
                        .collect()
                })
                .collect()
        })
        .collect();

    field
}
