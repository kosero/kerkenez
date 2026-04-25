pub mod buffer;
pub mod color;

pub mod draw_command;
pub mod light;
pub mod material;
pub mod pipeline;
pub mod post_processing;
pub mod shader;
pub mod texture;

use self::color::Color;
use self::draw_command::DrawCommand;
use self::light::SceneLights;
use self::material::{Material, MaterialId};
use crate::camera::Camera;
use crate::error::EngineError;
use crate::mesh::{Instance, Mesh, MeshType};
use glow::{Context, HasContext};
use std::collections::HashMap;
use std::rc::Rc;
use texture::{Texture, TextureId};
pub struct MeshBatch {
    gl: Rc<Context>,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
    instance_buffer: glow::Buffer,
    indices_count: i32,
    local_aabb: crate::mesh::primitives::AABB,
}

pub struct MainUniforms {
    pub view_projection: Option<glow::UniformLocation>,
    pub albedo_color: Option<glow::UniformLocation>,
    pub albedo_texture: Option<glow::UniformLocation>,
}

impl MainUniforms {
    pub fn new(gl: &glow::Context, program: glow::Program) -> Self {
        unsafe {
            Self {
                view_projection: gl.get_uniform_location(program, "u_ViewProjection"),
                albedo_color: gl.get_uniform_location(program, "u_AlbedoColor"),
                albedo_texture: gl.get_uniform_location(program, "u_Texture"),
            }
        }
    }
}

impl Drop for MeshBatch {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_buffer(self.ebo);
            self.gl.delete_buffer(self.instance_buffer);
        }
    }
}

// GL State Cache
/// Tracks the currently bound OpenGL objects to avoid redundant
/// state changes (state thrashing).
struct GlStateCache {
    current_vao: Option<glow::VertexArray>,
    current_program: Option<glow::Program>,
}

impl GlStateCache {
    fn new() -> Self {
        Self {
            current_vao: None,
            current_program: None,
        }
    }

    /// Bind VAO only if it differs from the currently bound one.
    unsafe fn bind_vao(&mut self, gl: &Context, vao: glow::VertexArray) {
        if self.current_vao != Some(vao) {
            unsafe {
                gl.bind_vertex_array(Some(vao));
            }
            self.current_vao = Some(vao);
        }
    }

    /// Use program only if it differs from the currently active one.
    unsafe fn use_program(&mut self, gl: &Context, program: glow::Program) {
        if self.current_program != Some(program) {
            unsafe {
                gl.use_program(Some(program));
            }
            self.current_program = Some(program);
        }
    }

    /// Invalidate all cached state (e.g. after post-processing pass).
    fn invalidate(&mut self) {
        self.current_vao = None;
        self.current_program = None;
    }
}

pub struct Renderer {
    gl: Rc<Context>,
    program: glow::Program,
    pub camera: Camera,

    batches: HashMap<MeshType, MeshBatch>,
    materials: HashMap<MaterialId, Material>,

    // Texture handle system: path → id for dedup, id → GPU texture for runtime
    textures: Vec<Texture>,
    texture_path_index: HashMap<String, TextureId>,

    state_cache: GlStateCache,
    pub post_processing: post_processing::PostProcessingManager,
    pub lights: SceneLights,
    next_material_id: usize,
    white_texture_id: TextureId,
    uniforms: MainUniforms,
    batch_buffer: HashMap<(MeshType, MaterialId), Vec<Instance>>,
    render_queue: Vec<DrawCommand>,
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
        }
    }
}

/// Grouping shared resources to satisfy Clippy's argument count limit
struct DrawResources<'a> {
    gl: &'a glow::Context,
    batches: &'a HashMap<MeshType, MeshBatch>,
    materials: &'a HashMap<MaterialId, Material>,
    textures: &'a [Texture],
    uniforms: &'a MainUniforms,
    state_cache: &'a mut GlStateCache,
}

impl Renderer {
    pub fn new(gl: Rc<Context>, width: u32, height: u32) -> Result<Self, EngineError> {
        let program = unsafe {
            let program = gl
                .create_program()
                .map_err(EngineError::ResourceCreationError)?;
            let vert_src = include_str!("../../shaders/geometry.vert");
            let frag_src = include_str!("../../shaders/geometry.frag");
            shader::create_shaders(&gl, program, vert_src, frag_src)?;
            gl.use_program(Some(program));
            program
        };

        let post_processing = post_processing::PostProcessingManager::new(&gl, width, height)?;

        let uniforms = MainUniforms::new(&gl, program);

        let mut state = Self {
            gl,
            program,
            camera: Camera::new_perspective(45.0, (width as f32) / (height as f32), 0.1, 1000.0),
            batches: HashMap::new(),
            materials: HashMap::new(),
            textures: Vec::new(),
            texture_path_index: HashMap::new(),
            state_cache: GlStateCache::new(),
            post_processing,
            lights: SceneLights::default(),
            next_material_id: 1,
            white_texture_id: TextureId::new(0), // Placeholder
            uniforms,
            batch_buffer: HashMap::new(),
            render_queue: Vec::new(),
        };

        // Create default white texture
        let white_tex = Texture::white(&state.gl);
        state.textures.push(white_tex);
        state.white_texture_id = TextureId::new(0); // It's the first texture in the list
        // RH: camera at +Z looking toward -Z (origin)
        state.camera.set_position(glam::vec3(0.0, 0.0, 10.0));
        state.camera.update();

        // Pre-register basic meshes
        state.register_mesh(MeshType::Square, &Mesh::square())?;
        state.register_mesh(MeshType::Triangle, &Mesh::triangle())?;
        state.register_mesh(MeshType::Cube, &Mesh::cube())?;

        // Register default material
        state.register_material(
            MaterialId::new(0),
            Material::new("default", Color::WHITE, None),
        );

        Ok(state)
    }

    /// Load a texture by path (deduplicated) and return its handle.
    fn load_texture(&mut self, path: &str) -> TextureId {
        if let Some(&id) = self.texture_path_index.get(path) {
            return id;
        }
        let texture = Texture::load(&self.gl, path);
        let id = TextureId::new(self.textures.len());
        self.textures.push(texture);
        self.texture_path_index.insert(path.to_string(), id);
        id
    }

    pub fn register_material(&mut self, id: MaterialId, mut material: Material) {
        if let Some(path) = material.texture_path.take() {
            let tex_id = self.load_texture(&path);
            material.albedo_texture = Some(tex_id);
        } else {
            material.albedo_texture = Some(self.white_texture_id);
        }
        self.materials.insert(id, material);
    }

    pub fn add_material(&mut self, material: Material) -> MaterialId {
        let id = MaterialId::new(self.next_material_id);
        self.next_material_id += 1;
        self.register_material(id, material);
        id
    }

    pub fn set_ambient_light(&mut self, r: f32, g: f32, b: f32, intensity: f32) {
        self.lights.ambient_color = crate::renderer::color::Color::rgb(r, g, b).to_linear();
        self.lights.ambient_intensity = intensity;
    }

    pub fn set_ambient_color(&mut self, color: crate::renderer::color::Color, intensity: f32) {
        self.lights.ambient_color = color.to_linear();
        self.lights.ambient_intensity = intensity;
    }

    pub fn set_fog(&mut self, color: crate::renderer::color::Color, density: f32) {
        self.post_processing.settings.fog_enabled = true;
        self.post_processing.settings.fog_color = color.to_linear();
        self.post_processing.settings.fog_density = density;
    }

    pub fn set_directional_light(&mut self, light: crate::renderer::light::DirectionalLight) {
        self.lights.directional = Some(light);
    }

    pub fn add_light(&mut self, light: crate::renderer::light::PointLight) {
        self.lights.point_lights.push(light);
    }

    fn register_mesh(&mut self, mesh_type: MeshType, mesh: &Mesh) -> Result<(), EngineError> {
        let (vao, vbo, ebo) = buffer::setup_mesh_buffers(&self.gl, mesh)?;
        unsafe {
            self.gl.bind_vertex_array(Some(vao));

            pipeline::setup_pipeline(&self.gl);
            let instance_buffer = buffer::setup_instance_buffer(&self.gl)?;
            pipeline::setup_instancing(&self.gl);

            self.batches.insert(
                mesh_type,
                MeshBatch {
                    gl: self.gl.clone(),
                    vao,
                    vbo,
                    ebo,
                    instance_buffer,
                    indices_count: mesh.indices.len() as i32,
                    local_aabb: crate::mesh::primitives::AABB {
                        min: mesh.bounding_box.min,
                        max: mesh.bounding_box.max,
                    },
                },
            );
        }
        Ok(())
    }

    pub fn begin_drawing(&mut self) {
        self.render_queue.clear();

        self.camera.update();

        unsafe {
            self.post_processing.begin(&self.gl);
            self.state_cache.invalidate();
            self.setup_frame();
        }
    }

    pub fn draw(&mut self, command: DrawCommand) {
        self.render_queue.push(command);
    }

    pub fn end_drawing(&mut self, time: f32) {
        unsafe {
            let queue = std::mem::take(&mut self.render_queue);
            self.prepare_batches(&queue);
            self.render_queue = queue;

            let mut resources = DrawResources {
                gl: &self.gl,
                batches: &self.batches,
                materials: &self.materials,
                textures: &self.textures,
                uniforms: &self.uniforms,
                state_cache: &mut self.state_cache,
            };

            for ((mesh_type, material_id), instances) in &self.batch_buffer {
                if !instances.is_empty() {
                    Self::draw_batch_internal(&mut resources, *mesh_type, *material_id, instances);
                }
            }

            self.state_cache.invalidate();

            let width = self.post_processing.width();
            let height = self.post_processing.height();
            self.post_processing
                .end(&self.gl, width, height, &self.camera, &self.lights, time);
        }
    }

    unsafe fn setup_frame(&mut self) {
        unsafe {
            self.state_cache.use_program(&self.gl, self.program);
            self.gl.clear_color(0.1, 0.1, 0.1, 1.0);
            self.gl.clear_depth_f32(1.0);
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            let vp = self.camera.view_projection_matrix();
            self.gl.uniform_matrix_4_f32_slice(
                self.uniforms.view_projection.as_ref(),
                false,
                &vp.to_cols_array(),
            );
        }
    }

    fn prepare_batches(&mut self, render_queue: &[DrawCommand]) {
        // Clear all vectors but keep capacity to avoid re-allocation
        for vec in self.batch_buffer.values_mut() {
            vec.clear();
        }

        let frustum = self.camera.frustum();

        for cmd in render_queue {
            let mesh_batch = match self.batches.get(&cmd.mesh_type) {
                Some(b) => b,
                None => continue,
            };

            let model_matrix = glam::Mat4::from_translation(cmd.position)
                * glam::Mat4::from_quat(cmd.rotation)
                * glam::Mat4::from_scale(cmd.scale);

            // Frustum Culling
            let world_aabb = mesh_batch.local_aabb.transform(&model_matrix);
            if !frustum.contains_aabb(&world_aabb) {
                continue;
            }

            self.batch_buffer
                .entry((cmd.mesh_type, cmd.material))
                .or_default()
                .push(Instance {
                    model_matrix,
                    tint: cmd.tint.to_vec4(),
                });
        }
    }

    unsafe fn draw_batch_internal(
        res: &mut DrawResources,
        mesh_type: MeshType,
        material_id: MaterialId,
        instances: &[Instance],
    ) {
        unsafe {
            if let (Some(batch), Some(material)) =
                (res.batches.get(&mesh_type), res.materials.get(&material_id))
            {
                res.gl.uniform_4_f32(
                    res.uniforms.albedo_color.as_ref(),
                    material.albedo_color.r,
                    material.albedo_color.g,
                    material.albedo_color.b,
                    material.albedo_color.a,
                );

                if let Some(tex_id) = &material.albedo_texture {
                    res.textures[tex_id.index()].bind_at(
                        res.gl,
                        res.uniforms.albedo_texture.as_ref(),
                        0,
                    );
                }

                // State-cached VAO bind — skips if already bound
                res.state_cache.bind_vao(res.gl, batch.vao);
                res.gl
                    .bind_buffer(glow::ARRAY_BUFFER, Some(batch.instance_buffer));

                // Buffer Orphaning: allocate with null first, then sub_data.
                let byte_len = std::mem::size_of_val(instances);
                res.gl
                    .buffer_data_size(glow::ARRAY_BUFFER, byte_len as i32, glow::DYNAMIC_DRAW);
                res.gl.buffer_sub_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    0,
                    bytemuck::cast_slice(instances),
                );

                pipeline::draw_instanced(res.gl, batch.indices_count, instances.len() as i32);
            }
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            unsafe {
                self.gl.viewport(0, 0, width as i32, height as i32);
            }
            self.camera.resize(width as f32, height as f32);
            self.post_processing
                .resize(&self.gl, width as i32, height as i32);
        }
    }
}
