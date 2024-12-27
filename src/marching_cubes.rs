use crate::data;
use crate::extras::Vertex;

pub fn generate_marching_cubes(
    grid_size: usize,
    scalar_field: &Vec<Vec<Vec<f32>>>,
    threshold: f32,
) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Iterate through each cell in the grid
    for x in 0..grid_size - 1 {
        for y in 0..grid_size - 1 {
            for z in 0..grid_size - 1 {
                let mut cube_index = 0;
                let mut corner_values = [0.0; 8];

                for i in 0..8 {
                    let corner_x = x + (i & 1);
                    let corner_y = y + ((i >> 1) & 1);
                    let corner_z = z + ((i >> 2) & 1);
                    corner_values[i] = scalar_field[corner_x][corner_y][corner_z];
                    if corner_values[i] < threshold {
                        cube_index |= 1 << i;
                    }
                }

                if data::EDGE_MASKS[cube_index] == 0 {
                    continue;
                }

                let mut edge_vertices = [[0.0; 3]; 12];
                for i in 0..12 {
                    if (data::EDGE_MASKS[cube_index] & (1 << i)) != 0 {
                        let [v1, v2] = data::EDGE_VERTEX_INDICES[i];
                        edge_vertices[i] = interpolate_vertex(
                            corner_values[v1],
                            corner_values[v2],
                            threshold,
                            corner_position(v1, x, y, z),
                            corner_position(v2, x, y, z),
                        );
                    }
                }

                let mut tri = 0;
                while data::TRIANGULATION_TABLE[cube_index][tri] != -1 {
                    let mut triangle_indices = Vec::new();
                    for j in 0..3 {
                        let edge_index = data::TRIANGULATION_TABLE[cube_index][tri + j] as usize;
                        let vertex_position = edge_vertices[edge_index];

                        let u = vertex_position[0] / 16.0 as f32;
                        let v = vertex_position[2] / 16.0 as f32;

                        // Create the vertex with the calculated texture coordinates
                        let vertex = Vertex {
                            pos: vertex_position,
                            tex_coords: [u, v], // The calculated UVs
                        };

                        let index = vertices.len() as u32;
                        vertices.push(vertex);
                        triangle_indices.push(index);
                    }

                    indices.extend(triangle_indices);
                    tri += 3;
                }
            }
        }
    }

    (vertices, indices)
}

fn interpolate_vertex(
    value1: f32,
    value2: f32,
    threshold: f32,
    position1: [f32; 3],
    position2: [f32; 3],
) -> [f32; 3] {
    let t = (threshold - value1) / (value2 - value1);
    [
        position1[0] + t * (position2[0] - position1[0]),
        position1[1] + t * (position2[1] - position1[1]),
        position1[2] + t * (position2[2] - position1[2]),
    ]
}

fn corner_position(corner: usize, x: usize, y: usize, z: usize) -> [f32; 3] {
    [
        (x + (corner & 1)) as f32,
        (y + ((corner >> 1) & 1)) as f32,
        (z + ((corner >> 2) & 1)) as f32,
    ]
}
