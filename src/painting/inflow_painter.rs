use crate::{
    coordinate_frame::affine_device_from_simulation,
    field::RgbaField,
    inflow::{Inflow, InflowPattern},
    math::{affine_map::AffineMap, matrix2::Matrix2, rect::Rect},
    painting::{
        gl_buffer::{GlBuffer, GlBufferTarget, GlVertexArrayObject},
        gl_texture::{Filter, GlTexture, Wrap},
        shader::{Shader, VertexAttribDesc},
        utils::RECT_TRIANGLE_INDICES,
    },
};
use ahash::HashMap;
use glow::HasContext;
use std::mem::offset_of;

#[derive(Debug, Clone, Copy)]
struct InflowVertex {
    pub position: [f32; 2],
}

pub struct InflowPainter {
    shader: Shader,
    array_buffer: GlBuffer<InflowVertex>,
    element_buffer: GlBuffer<u32>,
    vertex_array: GlVertexArrayObject,
    pattern_textures: HashMap<InflowPattern, GlTexture>,
}

impl InflowPainter {
    fn pattern_bitmap_bytes(pattern: InflowPattern) -> &'static [u8] {
        match pattern {
            InflowPattern::Uniform => include_bytes!("textures/pattern_uniform.png"),
            InflowPattern::VerticalStripes => {
                include_bytes!("textures/pattern_stripes_vertical.png")
            }
            InflowPattern::HorizontalStripes => {
                include_bytes!("textures/pattern_stripes_horizontal.png")
            }
            InflowPattern::DiagonalStripes => {
                include_bytes!("textures/pattern_stripes_diagonal.png")
            }
            InflowPattern::Noise => include_bytes!("textures/pattern_noise.png"),
        }
    }

    pub unsafe fn new(gl: &glow::Context) -> Self {
        let vs_source = include_str!("shaders/inflow.vert");
        let fs_source = include_str!("shaders/inflow.frag");
        let shader = Shader::from_source(gl, &vs_source, &fs_source);

        // Create vertex, index buffers and assign to shader
        let array_buffer = GlBuffer::new(gl, GlBufferTarget::ArrayBuffer);
        let element_buffer = GlBuffer::new(gl, GlBufferTarget::ElementArrayBuffer);
        let vertex_array = GlVertexArrayObject::new(gl);

        vertex_array.bind(gl);
        array_buffer.bind(gl);
        element_buffer.bind(gl);

        let size = size_of::<InflowVertex>();
        shader.assign_attribute_f32(
            gl,
            "in_simulation_position",
            &VertexAttribDesc::VEC2,
            offset_of!(InflowVertex, position) as i32,
            size as i32,
        );

        let pattern_textures = InflowPattern::ALL
            .into_iter()
            .map(|pattern| {
                let bytes = Self::pattern_bitmap_bytes(pattern);
                let image = RgbaField::load_from_memory(bytes).unwrap();
                let texture =
                    GlTexture::from_srgba_bitmap(gl, &image, Filter::Linear, Wrap::Repeat);
                (pattern, texture)
            })
            .collect();

        Self {
            shader,
            array_buffer,
            element_buffer,
            vertex_array,
            pattern_textures,
        }
    }

    pub unsafe fn draw(
        &mut self,
        gl: &glow::Context,
        inflow: &Inflow,
        simulation_bounds: Rect<f64>,
        time: f64,
    ) {
        let rect = inflow.rect().padded(1.0);
        let vertices = rect.corners().map(|corner| InflowVertex {
            position: corner.as_f32().to_array(),
        });
        let indices = RECT_TRIANGLE_INDICES;

        gl.disable(glow::BLEND);

        self.vertex_array.bind(gl);
        self.array_buffer.buffer_data(gl, &vertices);
        self.element_buffer.buffer_data(gl, &indices);

        self.shader.use_program(gl);

        // Inflow moves in inflow.direction at inflow.speed
        let simulation_from_inflow = AffineMap::new(
            Matrix2::orthogonal(inflow.direction),
            inflow.center + inflow.direction * inflow.speed * time,
        );
        let inflow_from_simulation = simulation_from_inflow.inv();
        let mut uv_from_simulation =
            AffineMap::uniform_scaling(inflow.pattern_scale) * inflow_from_simulation;

        // Texture lookup is repeating so we can use uv mod 1. We make the translation smaller by
        // subtracting a multiple of 1 to avoid f32 inaccuracies.
        uv_from_simulation.constant =
            uv_from_simulation.constant - uv_from_simulation.constant.round();

        self.shader
            .uniform(gl, "uv_from_simulation", &uv_from_simulation);

        self.shader.uniform(
            gl,
            "device_from_simulation",
            &affine_device_from_simulation(simulation_bounds),
        );

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(
            glow::TEXTURE_2D,
            Some(self.pattern_textures[&inflow.pattern].id),
        );
        self.shader.uniform(gl, "noise_texture", 0i32);

        self.shader.uniform(
            gl,
            "color_a",
            // pass linear RGB
            inflow.color_a.to_f32().srgb_to_linear().to_array(),
            // pass sRGB
            // inflow.color_a.to_f32().to_array(),
        );
        self.shader.uniform(
            gl,
            "color_b",
            // pass linear RGB
            inflow.color_b.to_f32().srgb_to_linear().to_array(),
            // pass sRGB
            // inflow.color_b.to_f32().to_array(),
        );

        gl.draw_elements(
            glow::TRIANGLES,
            self.element_buffer.len() as i32,
            glow::UNSIGNED_INT,
            0,
        );
    }
}
