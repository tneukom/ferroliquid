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
    pub point_size: f64,
}

impl Default for ParticlePainterSettings {
    fn default() -> Self {
        Self { point_size: 20.0 }
    }
}

#[derive(Debug, Clone, Copy)]
struct ParticleVertex {
    pub position: [f32; 2],
    pub previous_position: [f32; 2],
}

pub struct ParticlePainter {
    shader: Shader,
    dot_shader: Shader,
    array_buffer: GlBuffer<ParticleVertex>,
    vertex_array: GlVertexArrayObject,
}

impl ParticlePainter {
    pub unsafe fn new(gl: &glow::Context) -> Self {
        let shader = {
            let vs_source = include_str!("shaders/particle.vert");
            let fs_source = include_str!("shaders/particle.frag");
            Shader::from_source(gl, &vs_source, &fs_source)
        };

        let dot_shader = {
            let vs_source = include_str!("shaders/particle_dot.vert");
            let fs_source = include_str!("shaders/particle_dot.frag");
            Shader::from_source(gl, &vs_source, &fs_source)
        };

        // Create vertex, index buffers and assign to shader
        let array_buffer = GlBuffer::new(gl, GlBufferTarget::ArrayBuffer);
        let vertex_array = GlVertexArrayObject::new(gl);

        vertex_array.bind(gl);
        array_buffer.bind(gl);

        let size = size_of::<ParticleVertex>();
        shader.assign_attribute_f32(
            gl,
            "in_simulation_position",
            &VertexAttribDesc::VEC2,
            offset_of!(ParticleVertex, position) as i32,
            size as i32,
        );
        shader.assign_attribute_f32(
            gl,
            "in_simulation_previous_position",
            &VertexAttribDesc::VEC2,
            offset_of!(ParticleVertex, previous_position) as i32,
            size as i32,
        );

        dot_shader.assign_attribute_f32(
            gl,
            "in_simulation_position",
            &VertexAttribDesc::VEC2,
            offset_of!(ParticleVertex, position) as i32,
            size as i32,
        );

        array_buffer.unbind(gl);

        Self {
            shader,
            dot_shader,
            array_buffer,
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
        self.array_buffer.buffer_data(gl, &vertices);
    }

    pub unsafe fn draw_particles(
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

        self.shader.use_program(gl);
        self.shader.uniform(gl, "point_size", settings.point_size);
        self.shader.uniform(
            gl,
            "device_from_simulation",
            &affine_device_from_simulation(simulation_bounds),
        );

        gl.draw_arrays(glow::POINTS, 0, self.array_buffer.len() as i32);
    }

    pub unsafe fn draw_particle_dots(&self, gl: &glow::Context, simulation_bounds: Rect<f64>) {
        // Draw call
        gl.enable(glow::PROGRAM_POINT_SIZE);
        gl.disable(glow::BLEND);

        self.vertex_array.bind(gl);

        self.dot_shader.use_program(gl);
        self.dot_shader.uniform(gl, "point_size", 2.0);
        self.dot_shader.uniform(
            gl,
            "device_from_simulation",
            &affine_device_from_simulation(simulation_bounds),
        );

        gl.draw_arrays(glow::POINTS, 0, self.array_buffer.len() as i32);
    }
}
