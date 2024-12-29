use crate::camera::Camera;
use crate::extras::load_image_bytes;
use crate::marching_cubes::generate_marching_cubes;
use crate::scalar_generator::generate_scalar_field;
use crate::shader;
use image::{ImageBuffer, Rgba};
use miniquad::*;
use std::time::Instant;
use window::screen_size;

pub struct Stage {
    pipeline: Pipeline,
    bindings: Bindings,
    ctx: Box<dyn RenderingBackend>,
    index_count: i32,
    camera: Camera,
    render_texture: TextureId,
    render_pass: RenderPass,
    last_frame_time: Instant,
}

impl Stage {
    pub fn new() -> Stage {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        // Trap the mouse and hide the cursor
        window::show_mouse(false);

        // Generate scalar field and mesh
        let grid_size = 512;
        let scalar_field = generate_scalar_field(grid_size, vec![1, 1, 1]);
        println!("Scalar field generated!");
        let threshold = 0.9;
        let (vertices, indices) = generate_marching_cubes(grid_size, &scalar_field, threshold);
        println!("Generated Mesh!");

        // Load texture
        let (image_data, width, height) = load_image_bytes("./assets/textures/grass.png");

        let texture = ctx.new_texture(
            TextureAccess::Static,
            TextureSource::Bytes(&image_data),
            TextureParams {
                width: width,
                height: height,
                format: TextureFormat::RGBA8,
                wrap: TextureWrap::Repeat,
                kind: TextureKind::Texture2D,
                min_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
                mipmap_filter: MipmapFilterMode::Nearest,
                allocate_mipmaps: true,
                sample_count: 1,
            },
        );

        ctx.texture_generate_mipmaps(texture);

        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer.clone()],
            index_buffer: index_buffer.clone(),
            images: vec![texture],
        };

        let render_texture = ctx.new_texture(
            TextureAccess::Static,
            TextureSource::Empty,
            TextureParams {
                width: 1024,
                height: 1024,
                format: TextureFormat::RGBA8,
                wrap: TextureWrap::Clamp,
                kind: TextureKind::Texture2D,
                min_filter: FilterMode::Linear,
                mag_filter: FilterMode::Linear,
                mipmap_filter: MipmapFilterMode::None,
                allocate_mipmaps: false,
                sample_count: 1,
            },
        );

        let render_pass = ctx.new_render_pass(render_texture, None);

        let shader = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: include_str!("../assets/shaders/default/vertex.glsl"),
                    fragment: include_str!("../assets/shaders/default/fragment.glsl"),
                },
                ShaderMeta {
                    images: vec!["tex".to_string()],
                    uniforms: UniformBlockLayout {
                        uniforms: vec![UniformDesc::new("mvp", UniformType::Mat4)],
                    },
                },
            )
            .unwrap();

        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("in_pos", VertexFormat::Float3),
                VertexAttribute::new("in_tex_coord", VertexFormat::Float2),
            ],
            shader,
            PipelineParams {
                depth_test: Comparison::Less,
                depth_write: true,
                primitive_type: PrimitiveType::Triangles,
                ..Default::default()
            },
        );

        Stage {
            pipeline,
            bindings,
            ctx,
            index_count: indices.len() as i32,
            camera: Camera::new(),
            render_texture,
            render_pass,
            last_frame_time: Instant::now(),
        }
    }

    fn calculate_ortho_mvp(&self) -> [[f32; 4]; 4] {
        let ortho_projection =
            glam::Mat4::orthographic_rh_gl(-200.0, 200.0, -200.0, 200.0, 0.1, 1000.0);
        let ortho_view = glam::Mat4::look_at_rh(
            glam::Vec3::new(10.0, 500.0, 10.0),
            glam::Vec3::new(0.0, 0.0, 0.0),
            glam::Vec3::new(0.0, 1.0, 0.0),
        );
        (ortho_projection * ortho_view).to_cols_array_2d()
    }

    fn save_texture_to_png(&mut self) {
        let width = 1024;
        let height = 1024;

        self.ctx.commit_frame();

        let mut texture_data = vec![0u8; (width * height * 4) as usize];

        self.ctx
            .texture_read_pixels(self.render_texture, &mut texture_data);

        let non_zero_pixels = texture_data.iter().filter(|&&b| b > 0).count();
        println!("Non-zero pixels: {}", non_zero_pixels);

        let buffer: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(width, height, texture_data)
            .expect("Failed to create image buffer from texture data");

        buffer
            .save("output.png")
            .expect("Failed to save texture as PNG");

        println!("Texture saved as output.png");
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {
        self.camera.process_input();
    }

    fn draw(&mut self) {
        let current_time = Instant::now();
        let frame_time = current_time.duration_since(self.last_frame_time);
        self.last_frame_time = current_time;

        let aspect_ratio = screen_size().0 as f32 / screen_size().1 as f32;

        let view = glam::Mat4::look_at_rh(
            self.camera.position,
            self.camera.position + self.camera.front,
            self.camera.up,
        );
        let projection =
            glam::Mat4::perspective_rh_gl(45.0_f32.to_radians(), aspect_ratio, 0.1, 2048.0);

        let mvp = projection * view;
        let ortho_mvp = self.calculate_ortho_mvp();

        // Render scene to render texture
        self.ctx.begin_pass(
            Some(self.render_pass),
            PassAction::clear_color(0.0, 0.0, 0.0, 1.0),
        );
        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx.apply_bindings(&self.bindings);
        self.ctx
            .apply_uniforms(UniformsSource::table(&shader::UniformsDefault {
                mvp: ortho_mvp,
            }));
        self.ctx.draw(0, self.index_count, 1);
        self.ctx.end_render_pass();

        // Render scene to screen
        self.ctx
            .begin_default_pass(PassAction::clear_color(0.4, 0.45, 0.7, 1.0));
        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx.apply_bindings(&self.bindings);
        self.ctx
            .apply_uniforms(UniformsSource::table(&shader::UniformsDefault {
                mvp: mvp.to_cols_array_2d(),
            }));
        self.ctx.draw(0, self.index_count, 1);
        self.ctx.end_render_pass();

        self.ctx.commit_frame();

        let frame_time_ms = frame_time.as_secs_f64() * 1000.0;
        let fps = 1000.0 / frame_time_ms;

        println!("Frame time: {:.2}ms, FPS: {:.2}", frame_time_ms, fps);
    }

    fn key_down_event(&mut self, keycode: KeyCode, _mods: KeyMods, _repeat: bool) {
        self.camera.keys[keycode as usize] = true;

        if keycode == KeyCode::Space {
            self.save_texture_to_png();
        }
    }

    fn key_up_event(&mut self, keycode: KeyCode, _mods: KeyMods) {
        self.camera.keys[keycode as usize] = false;
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        let (last_x, last_y) = self.camera.last_mouse_pos;

        if last_x != 0.0 || last_y != 0.0 {
            let xoffset = x - last_x;
            let yoffset = last_y - y;
            self.camera.process_mouse(xoffset, yoffset);
        }

        self.camera.last_mouse_pos = (x, y);
    }
}
