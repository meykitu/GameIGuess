use image::Rgba;
use miniquad::*;
use window::screen_size;
use image::ImageBuffer;
use crate::camera::Camera;
use crate::marching_cubes::generate_marching_cubes;
use crate::scalar_generator::generate_scalar_field;
use crate::shader;
use crate::extras::load_image_bytes;

pub struct Stage {
    pipeline: Pipeline,
    shadow_pipeline: Pipeline,
    bindings: Bindings,
    shadow_bindings: Bindings,
    ctx: Box<dyn RenderingBackend>,
    index_count: i32,
    camera: Camera,
    shadow_map: TextureId,
    shadow_pass: RenderPass,
}

impl Stage {
    pub fn new() -> Stage {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        // Stop being anoying and show the mouse cursor you dumb window library >:( 
        window::show_mouse(true);

        // Shadow map creation

        let shadow_map_size = 2048;
        let shadow_map = ctx.new_render_texture(
            TextureParams {
                width: shadow_map_size,
                height: shadow_map_size,
                format: TextureFormat::Depth32,
                wrap: TextureWrap::Clamp,
                kind: TextureKind::Texture2D,
                min_filter: FilterMode::Nearest,
                mag_filter: FilterMode::Nearest,
                mipmap_filter: MipmapFilterMode::None,
                allocate_mipmaps: false,
                sample_count: 1,
            },
        );

        let shadow_pass = ctx.new_render_pass(shadow_map, Some(shadow_map));

        // Generate scalar field and mesh
        let grid_size = 256;
        let scalar_field = generate_scalar_field(grid_size, vec![1, 1, 1]);
        println!("Scalar field generated!");
        let threshold = 0.5;
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
                mipmap_filter: MipmapFilterMode::Linear,
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

        let shadow_bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![shadow_map], // Add the shadow_map texture to the bindings
        };

        let shader = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: include_str!("../assets/shaders/default/vertex.glsl"),
                    fragment: include_str!("../assets/shaders/default/fragment.glsl"),
                },
                ShaderMeta {
                    images: vec!["tex".to_string(), "shadow_map".to_string()],
                    uniforms: UniformBlockLayout {
                        uniforms: vec![
                            UniformDesc::new("mvp", UniformType::Mat4),
                            UniformDesc::new("light_mvp", UniformType::Mat4),
                        ],
                    },
                }
            )
            .unwrap();

        let shadow_shader = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: include_str!("../assets/shaders/shadow/vertex.glsl"),
                    fragment: include_str!("../assets/shaders/shadow/fragment.glsl"),
                },
                ShaderMeta {
                    images: vec!["shadow_map".to_string()],
                    uniforms: UniformBlockLayout {
                        uniforms: vec![
                            UniformDesc::new("mvp", UniformType::Mat4),
                            UniformDesc::new("light_mvp", UniformType::Mat4),
                        ],
                    },
                }
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

        let shadow_pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[VertexAttribute::new("in_pos", VertexFormat::Float3)],
            shadow_shader,
            PipelineParams {
                depth_test: Comparison::Less,
                depth_write: true,
                primitive_type: PrimitiveType::Triangles,
                ..Default::default()
            },
        );
        
        Stage {
            pipeline,
            shadow_pipeline,
            bindings,
            shadow_bindings,
            ctx,
            index_count: indices.len() as i32,
            camera: Camera::new(),
            shadow_map,
            shadow_pass,
        }
    }

    fn save_shadow_map_to_image(&mut self) {

        let mut texture_data = vec![0u8; (2048 * 2048 * 4) as usize];

        self.ctx.texture_read_pixels(self.shadow_map, &mut texture_data);

        let width = 2048;
        let height = 2048;

        let buffer: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(width, height, texture_data)
            .expect("Failed to create image buffer from texture data");
    
        buffer
            .save("output.png")
            .expect("Failed to save texture as PNG");
        println!("Output saved!");
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {
        self.camera.process_input();
    }

    fn draw(&mut self) {
        let aspect_ratio = screen_size().0 as f32 / screen_size().1 as f32;

        let view = glam::Mat4::look_at_rh(self.camera.position, self.camera.position + self.camera.front, self.camera.up);
        let projection = glam::Mat4::perspective_rh_gl(
            45.0_f32.to_radians(),
            aspect_ratio,
            0.1,
            2048.0,
        );

        let mvp = projection * view;

        let light_pos = glam::Vec3::new(10.0, 10.0, 10.0);
        let light_target = glam::Vec3::new(0.0, 0.0, 0.0);
        let light_up = glam::Vec3::new(0.0, 1.0, 0.0);
        let light_view = glam::Mat4::look_at_rh(light_pos, light_target, light_up);
        let light_projection = glam::Mat4::orthographic_rh_gl(-20.0, 20.0, -20.0, 20.0, 0.1, 2048.0);

        let light_mvp = light_projection * light_view;

        // Render shadow map
        self.ctx.begin_pass(Some(self.shadow_pass), PassAction::Clear {
            color: None,        // Don't clear the color buffer
            depth: Some(1.0),   // Clear the depth buffer to 1.0 (far depth)
            stencil: None,      // Don't clear the stencil buffer
        });
        
        self.ctx.apply_pipeline(&self.shadow_pipeline);
        self.ctx.apply_bindings(&self.shadow_bindings);
        self.ctx.apply_uniforms(UniformsSource::table(&shader::UniformsShadow {
            mvp: light_mvp.to_cols_array_2d(),
            light_mvp: light_mvp.to_cols_array_2d(),
        }));
        self.ctx.draw(0, self.index_count, 1);
        self.ctx.end_render_pass();


        // Render scene with shadows
        self.ctx.begin_default_pass(PassAction::clear_color(0.2, 0.25, 0.7, 1.0));
        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx.apply_bindings(&Bindings {
            vertex_buffers: vec![self.bindings.vertex_buffers[0].clone()],
            index_buffer: self.bindings.index_buffer.clone(),
            images: vec![self.bindings.images[0], self.shadow_map],
        });
        self.ctx.apply_uniforms(UniformsSource::table(&shader::UniformsShadow {
            mvp: mvp.to_cols_array_2d(),
            light_mvp: light_mvp.to_cols_array_2d(),
        }));
        self.ctx.draw(0, self.index_count, 1);
        self.ctx.end_render_pass();

        self.ctx.commit_frame();

        //self.save_shadow_map_to_image();
    }

    fn key_down_event(&mut self, keycode: KeyCode, _mods: KeyMods, _repeat: bool) {
        self.camera.keys[keycode as usize] = true;

        if keycode == KeyCode::Space {
            self.save_shadow_map_to_image();
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
