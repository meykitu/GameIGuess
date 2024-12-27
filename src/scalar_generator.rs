use noise::{Perlin, NoiseFn};

pub fn generate_scalar_field(grid_size: usize, position: Vec<i32>) -> Vec<Vec<Vec<f32>>> {
    let noise = Perlin::new(1024);

    let mut field = vec![vec![vec![0.0; grid_size]; grid_size]; grid_size];

    let global_scale = 1.2;

    let scale = 0.01 * global_scale;
    let falloff = 0.01 * global_scale;

    for x in 0..grid_size {
        for y in 0..grid_size {
            for z in 0..grid_size {

                let noise_value = noise.get([(x as f64 + position[0] as f64) * scale, (y as f64 + position[1] as f64) * scale, (z as f64 + position[2] as f64) * scale]);

                let final_value = noise_value - ((y as f64 - 150.0 * global_scale) * falloff);

                field[x][y][z] = final_value as f32;
            }
        }
    }

    field
}
