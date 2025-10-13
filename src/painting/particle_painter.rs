use crate::{
    coordinate_frame::affine_device_from_simulation,
    math::rect::Rect,
    painting::{
        gl_buffer::{GlBuffer, GlBufferTarget, GlVertexArrayObject},
        shader::{Shader, VertexAttribDesc},
        utils::check_error,
    },
    simulation::Particle,
};
use glow::HasContext;
use std::mem::{offset_of, size_of};

#[derive(Debug, Clone, Copy)]
struct ParticleVertex {
    pub position: [f32; 2],
}

pub struct ParticlePainter {
    shader: Shader,
    array_buffer: GlBuffer<ParticleVertex>,
    vertex_array: GlVertexArrayObject,
}

impl ParticlePainter {
    pub unsafe fn new(gl: &glow::Context) -> Self {
        let vs_source = include_str!("shaders/particle.vert");
        let fs_source = include_str!("shaders/particle.frag");
        let shader = Shader::from_source(gl, &vs_source, &fs_source);

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

        array_buffer.unbind(gl);

        Self {
            shader,
            array_buffer,
            vertex_array,
        }
    }

    pub unsafe fn draw_particles(
        &self,
        gl: &glow::Context,
        particles: &[Particle],
        simulation_bounds: Rect<f64>,
    ) {
        let vertices: Vec<_> = particles
            .iter()
            .map(|particle| ParticleVertex {
                position: particle.position.as_f32().to_array(),
            })
            .collect();

        // Draw call
        gl.enable(glow::PROGRAM_POINT_SIZE);
        gl.enable(glow::BLEND);
        gl.blend_func(glow::ONE, glow::ONE);
        gl.blend_equation(glow::FUNC_ADD);

        self.vertex_array.bind(gl);
        self.array_buffer.buffer_data(gl, &vertices);

        self.shader.use_program(gl);
        self.shader.uniform(gl, "point_size", 20.0);
        self.shader.uniform(
            gl,
            "device_from_simulation",
            &affine_device_from_simulation(simulation_bounds),
        );

        gl.draw_arrays(glow::POINTS, 0, vertices.len() as i32);
    }
}
