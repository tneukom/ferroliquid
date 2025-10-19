use crate::painting::{
    gl_buffer::{GlBuffer, GlBufferTarget, GlVertexArrayObject},
    shader::{Shader, VertexAttribDesc},
};
use glow::HasContext;
use std::mem::offset_of;

#[derive(Debug, Clone, Copy)]
struct EffectVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

pub struct EffectPainter {
    pub shader: Shader,
    array_buffer: GlBuffer<EffectVertex>,
    vertex_array: GlVertexArrayObject,
}

impl EffectPainter {
    pub unsafe fn new(gl: &glow::Context, vs_source: &str, fs_source: &str) -> Self {
        let shader = Shader::from_source(gl, &vs_source, &fs_source);

        // 2           3
        // ┌──────────┐
        // │          │
        // └──────────┘
        // 0           1
        let vertices = [
            EffectVertex {
                position: [-1.0, -1.0],
                uv: [0.0, 0.0],
            },
            EffectVertex {
                position: [1.0, -1.0],
                uv: [1.0, 0.0],
            },
            EffectVertex {
                position: [-1.0, 1.0],
                uv: [0.0, 1.0],
            },
            EffectVertex {
                position: [1.0, 1.0],
                uv: [1.0, 1.0],
            },
        ];

        // Create vertex, index buffers and assign to shader
        let mut array_buffer = GlBuffer::new(gl, GlBufferTarget::ArrayBuffer);
        let vertex_array = GlVertexArrayObject::new(gl);

        vertex_array.bind(gl);
        array_buffer.bind(gl);

        array_buffer.buffer_data(gl, &vertices);

        let size = size_of::<EffectVertex>();
        shader.assign_attribute_f32(
            gl,
            "in_device_position",
            &VertexAttribDesc::VEC2,
            offset_of!(EffectVertex, position) as i32,
            size as i32,
        );

        shader.assign_attribute_f32(
            gl,
            "in_uv",
            &VertexAttribDesc::VEC2,
            offset_of!(EffectVertex, uv) as i32,
            size as i32,
        );

        array_buffer.unbind(gl);

        Self {
            shader,
            array_buffer,
            vertex_array,
        }
    }

    pub unsafe fn setup_draw(&self, gl: &glow::Context) {
        self.vertex_array.bind(gl);
        self.shader.use_program(gl);
    }

    pub unsafe fn draw(&self, gl: &glow::Context) {
        gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
    }
}
