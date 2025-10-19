use crate::{
    coordinate_frame::affine_device_from_simulation,
    math::{rect::Rect, rgba8::Rgba8},
    painting::{
        gl_buffer::{GlBuffer, GlBufferTarget, GlVertexArrayObject},
        shader::{Shader, VertexAttribDesc},
        utils::RECT_TRIANGLE_INDICES,
    },
};
use glow::HasContext;
use std::mem::offset_of;

#[derive(Debug, Clone, Copy)]
struct RectVertex {
    pub position: [f32; 2],
    pub color: Rgba8,
}

pub struct RectPainter {
    shader: Shader,
    array_buffer: GlBuffer<RectVertex>,
    element_buffer: GlBuffer<u32>,
    vertex_array: GlVertexArrayObject,
}

impl RectPainter {
    pub unsafe fn new(gl: &glow::Context) -> Self {
        let vs_source = include_str!("shaders/rect.vert");
        let fs_source = include_str!("shaders/rect.frag");
        let shader = Shader::from_source(gl, &vs_source, &fs_source);

        // Create vertex, index buffers and assign to shader
        let array_buffer = GlBuffer::new(gl, GlBufferTarget::ArrayBuffer);
        let element_buffer = GlBuffer::new(gl, GlBufferTarget::ElementArrayBuffer);
        let vertex_array = GlVertexArrayObject::new(gl);

        vertex_array.bind(gl);
        array_buffer.bind(gl);
        element_buffer.bind(gl);

        let size = size_of::<RectVertex>();
        shader.assign_attribute_f32(
            gl,
            "in_simulation_position",
            &VertexAttribDesc::VEC2,
            offset_of!(RectVertex, position) as i32,
            size as i32,
        );
        shader.assign_attribute_f32(
            gl,
            "in_color",
            &VertexAttribDesc::RGBA8,
            offset_of!(RectVertex, color) as i32,
            size as i32,
        );

        Self {
            shader,
            array_buffer,
            element_buffer,
            vertex_array,
        }
    }

    pub unsafe fn draw(
        &mut self,
        gl: &glow::Context,
        rects: impl IntoIterator<Item = (Rect<f64>, Rgba8)>,
        simulation_bounds: Rect<f64>,
    ) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for (rect, color) in rects {
            for index in RECT_TRIANGLE_INDICES {
                indices.push(index + vertices.len() as u32);
            }

            for corner in rect.corners() {
                let vertex = RectVertex {
                    position: corner.as_f32().to_array(),
                    color,
                };
                vertices.push(vertex);
            }
        }

        gl.disable(glow::BLEND);

        self.vertex_array.bind(gl);
        self.array_buffer.buffer_data(gl, &vertices);
        self.element_buffer.buffer_data(gl, &indices);

        self.shader.use_program(gl);

        self.shader.uniform(
            gl,
            "device_from_simulation",
            &affine_device_from_simulation(simulation_bounds),
        );

        gl.draw_elements(
            glow::TRIANGLES,
            self.element_buffer.len() as i32,
            glow::UNSIGNED_INT,
            0,
        );
    }
}
