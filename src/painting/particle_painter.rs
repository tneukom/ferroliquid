use crate::{
    coordinate_frame::affine_device_from_simulation,
    math::rect::Rect,
    painting::{
        gl_buffer::{GlBuffer, GlBufferTarget, GlVertexArrayObject},
        shader::{Shader, VertexAttribDesc},
    },
    simulation::Particle,
};
use glow::HasContext;
use std::mem::{offset_of, size_of};

#[derive(Debug, Clone)]
pub struct ParticlePainterSettings {
    /// radius in simulation coordinates
    pub delta_point_radius: f64,

    /// radius in simulation coordinates
    pub distance_point_radius: f64,
}

impl Default for ParticlePainterSettings {
    fn default() -> Self {
        Self {
            delta_point_radius: 20.0,
            distance_point_radius: 11.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ParticleVertex {
    pub position: [f32; 2],
    pub previous_position: [f32; 2],
}

#[derive(Debug, Clone, Copy)]
struct QuadVertex {
    pub offset: [f32; 2],
}

/// All shaders use explicit attribute locations, so we can use one vertex array for all of them.
pub struct ParticlePainter {
    delta_shader: Shader,
    distance_shader: Shader,
    dot_shader: Shader,
    particle_vertex_buffer: GlBuffer<ParticleVertex>,
    quad_vertex_buffer: GlBuffer<QuadVertex>,
    vertex_array: GlVertexArrayObject,
}

impl ParticlePainter {
    pub unsafe fn new(gl: &glow::Context) -> Self {
        let delta_shader = {
            let vs_source = include_str!("shaders/particle.vert");
            let fs_source = include_str!("shaders/particle.frag");
            Shader::from_source(gl, &vs_source, &fs_source)
        };

        let distance_shader = {
            let vs_source = include_str!("shaders/particle_distance.vert");
            let fs_source = include_str!("shaders/particle_distance.frag");
            Shader::from_source(gl, &vs_source, &fs_source)
        };

        let dot_shader = {
            let vs_source = include_str!("shaders/particle_dot.vert");
            let fs_source = include_str!("shaders/particle_dot.frag");
            Shader::from_source(gl, &vs_source, &fs_source)
        };

        // 2           3
        // ┌──────────┐
        // │          │
        // └──────────┘
        // 0           1
        let quad_vertices = [
            QuadVertex {
                offset: [-1.0, -1.0],
            },
            QuadVertex {
                offset: [1.0, -1.0],
            },
            QuadVertex {
                offset: [-1.0, 1.0],
            },
            QuadVertex { offset: [1.0, 1.0] },
        ];

        let vertex_array = GlVertexArrayObject::new(gl);
        vertex_array.bind(gl);

        let particle_vertex_buffer = GlBuffer::new(gl, GlBufferTarget::ArrayBuffer);
        let mut quad_vertex_buffer = GlBuffer::new(gl, GlBufferTarget::ArrayBuffer);
        quad_vertex_buffer.buffer_data(gl, &quad_vertices);

        // Setup vertex array for delta shader
        quad_vertex_buffer.bind(gl);
        VertexAttribDesc::VEC2.assign_attribute_f32(
            gl,
            0,
            offset_of!(QuadVertex, offset) as i32,
            size_of::<QuadVertex>() as i32,
        );

        particle_vertex_buffer.bind(gl);
        VertexAttribDesc::VEC2.assign_attribute_f32(
            gl,
            1,
            offset_of!(ParticleVertex, position) as i32,
            size_of::<ParticleVertex>() as i32,
        );
        gl.vertex_attrib_divisor(1, 1);
        VertexAttribDesc::VEC2.assign_attribute_f32(
            gl,
            2,
            offset_of!(ParticleVertex, previous_position) as i32,
            size_of::<ParticleVertex>() as i32,
        );
        gl.vertex_attrib_divisor(2, 1);

        vertex_array.unbind(gl);

        Self {
            delta_shader,
            distance_shader,
            dot_shader,
            particle_vertex_buffer,
            quad_vertex_buffer,
            vertex_array,
        }
    }

    pub unsafe fn update_particles(&mut self, gl: &glow::Context, particles: &[Particle]) {
        let vertices: Vec<_> = particles
            .iter()
            .map(|particle| ParticleVertex {
                position: particle.position.as_f32().to_array(),
                previous_position: particle.previous_position.as_f32().to_array(),
            })
            .collect();

        self.vertex_array.bind(gl);
        self.particle_vertex_buffer.buffer_data(gl, &vertices);
    }

    pub unsafe fn draw_arrays_instanced(&self, gl: &glow::Context) {
        let instance_count = self.particle_vertex_buffer.len() as i32;
        let vertex_count = self.quad_vertex_buffer.len() as i32;
        gl.draw_arrays_instanced(glow::TRIANGLE_STRIP, 0, vertex_count, instance_count);
    }

    pub unsafe fn draw_delta(
        &self,
        gl: &glow::Context,
        simulation_bounds: Rect<f64>,
        settings: &ParticlePainterSettings,
    ) {
        // Draw call
        gl.enable(glow::PROGRAM_POINT_SIZE);
        gl.enable(glow::BLEND);
        gl.blend_func(glow::ONE, glow::ONE);
        gl.blend_equation(glow::FUNC_ADD);

        self.vertex_array.bind(gl);

        self.delta_shader.use_program(gl);
        self.delta_shader.uniform(
            gl,
            "device_from_simulation",
            &affine_device_from_simulation(simulation_bounds),
        );
        self.delta_shader
            .uniform(gl, "simulation_radius", settings.delta_point_radius);

        self.draw_arrays_instanced(gl);
        self.vertex_array.unbind(gl);
    }

    pub unsafe fn draw_distance(
        &self,
        gl: &glow::Context,
        simulation_bounds: Rect<f64>,
        settings: &ParticlePainterSettings,
    ) {
        gl.enable(glow::PROGRAM_POINT_SIZE);
        gl.enable(glow::BLEND);
        gl.blend_func(glow::ONE, glow::ONE);
        gl.blend_equation(glow::MAX);

        self.vertex_array.bind(gl);

        self.distance_shader.use_program(gl);
        self.distance_shader.uniform(
            gl,
            "device_from_simulation",
            &affine_device_from_simulation(simulation_bounds),
        );
        self.distance_shader
            .uniform(gl, "simulation_radius", settings.distance_point_radius);

        self.draw_arrays_instanced(gl);
        self.vertex_array.unbind(gl);
    }

    pub unsafe fn draw_particle_dots(&self, gl: &glow::Context, simulation_bounds: Rect<f64>) {
        // Draw call
        gl.enable(glow::PROGRAM_POINT_SIZE);
        gl.disable(glow::BLEND);

        self.vertex_array.bind(gl);

        self.dot_shader.use_program(gl);
        self.dot_shader.uniform(
            gl,
            "device_from_simulation",
            &affine_device_from_simulation(simulation_bounds),
        );
        self.dot_shader.uniform(gl, "simulation_radius", 0.33);

        self.draw_arrays_instanced(gl);
        self.vertex_array.unbind(gl);
    }
}
