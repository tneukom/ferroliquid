use crate::{
    math::{affine_map::AffineMap, point::Point, rect::Rect},
    painting::{
        gl_buffer::{GlBuffer, GlBufferTarget, GlVertexArrayObject},
        gl_texture::GlTexture,
        shader::{Shader, VertexAttribDesc},
        utils::RECT_TRIANGLE_INDICES,
    },
};
use glow::HasContext;
use std::mem::offset_of;

pub struct Sprite {
    /// Rectangle in the source texture in pixels
    pub bitmap_rect: Rect<i64>,

    /// Rectangle to paint in target coordinates, without padding
    pub target_rect: Rect<f64>,

    /// Counter-clockwise rotation in 90° steps
    pub rotation: i64,
}

/// Helper class to draw tilemaps using SpritePainter
/// Outer size = tile_size + padding
/// ┌────────────┬────────────┐
/// │   Padding  │   Padding  │
/// │  ┌──────┐  │  ┌──────┐  │
/// │  │      │  │  │      │  │
/// │  │Inner │  │  │Inner │  │
/// │  │      │  │  │      │  │
/// │  └──────┘  │  └──────┘  │
/// │            │            │
/// ├────────────┼────────────┤
/// │   Padding  │   Padding  │
/// │  ┌──────┐  │  ┌──────┐  │
/// │  │      │  │  │      │  │
/// │  │Inner │  │  │Inner │  │
/// │  │      │  │  │      │  │
/// │  └──────┘  │  └──────┘  │
/// │            │            │
/// └────────────┴────────────┘
pub struct TileSheet {
    pub tile_size: Point<i64>,

    pub tile_padding: Point<i64>,

    /// Number of tiles (columns, rows)
    pub size: Point<i64>,
}

impl TileSheet {
    pub fn padded_tile_size(&self) -> Point<i64> {
        self.tile_size + 2 * self.tile_padding
    }

    /// rect is inner rect, without padding.
    pub fn sprite(&self, tile_index: Point<i64>, target_rect: Rect<f64>, rotation: i64) -> Sprite {
        let texture_rect = Rect::low_size(
            tile_index * self.padded_tile_size(),
            self.padded_tile_size(),
        );

        // Add proportional padding to rect, such that
        // target_padding / target_rect.size = self.tile_padding / self.tile_size
        let target_padding =
            target_rect.size() * self.tile_padding.as_f64() / self.tile_size.as_f64();
        let padded_target_rect = target_rect.padded_xy(target_padding);

        Sprite {
            bitmap_rect: texture_rect,
            target_rect: padded_target_rect,
            rotation,
        }
    }

    pub fn pixel_size(&self) -> Point<i64> {
        self.padded_tile_size() * self.size
    }
}

#[derive(Debug, Clone, Copy)]
struct SpriteVertex {
    pub position: [f32; 2],

    /// In pixels
    pub bitmap_position: [f32; 2],
}

pub struct SpritePainter<Aux> {
    pub shader: Shader,
    array_buffer: GlBuffer<SpriteVertex>,
    pub aux_array_buffer: GlBuffer<Aux>,
    element_buffer: GlBuffer<u32>,
    vertex_array: GlVertexArrayObject,
}

impl<Aux: Copy> SpritePainter<Aux> {
    /// Assign vertex attribute binding for the aux buffer after new()
    pub unsafe fn new(gl: &glow::Context, vs_source: &str, fs_source: &str) -> Self {
        let shader = Shader::from_source(gl, &vs_source, &fs_source);

        // Create vertex, index buffers and assign to shader
        let array_buffer = GlBuffer::new(gl, GlBufferTarget::ArrayBuffer);
        let aux_array_buffer = GlBuffer::new(gl, GlBufferTarget::ArrayBuffer);
        let element_buffer = GlBuffer::new(gl, GlBufferTarget::ElementArrayBuffer);
        let vertex_array = GlVertexArrayObject::new(gl);

        vertex_array.bind(gl);
        element_buffer.bind(gl);

        array_buffer.bind(gl);
        let stride = size_of::<SpriteVertex>();
        shader.assign_attribute_f32(
            gl,
            "in_position",
            &VertexAttribDesc::VEC2,
            offset_of!(SpriteVertex, position) as i32,
            stride as i32,
        );
        shader.assign_attribute_f32(
            gl,
            "in_bitmap_position",
            &VertexAttribDesc::VEC2,
            offset_of!(SpriteVertex, bitmap_position) as i32,
            stride as i32,
        );

        aux_array_buffer.bind(gl);

        Self {
            shader,
            array_buffer,
            aux_array_buffer,
            element_buffer,
            vertex_array,
        }
    }

    /// Call this, then assign uniforms, then call draw
    /// * `device_from` - Coordinate transformation from sprite target coordinates into OpenGl
    ///   device coordinates.
    pub unsafe fn setup_draw(
        &mut self,
        gl: &glow::Context,
        sprites: impl IntoIterator<Item = (Sprite, Aux)>,
        bitmap_size: Point<i64>,
        device_from: AffineMap<f64>,
    ) {
        let mut vertices = Vec::new();
        let mut aux_vertices = Vec::new();
        let mut indices = Vec::new();
        for (sprite, aux) in sprites {
            for index in RECT_TRIANGLE_INDICES {
                indices.push(index + vertices.len() as u32);
            }

            let mut bitmap_corners = sprite.bitmap_rect.corners();
            bitmap_corners.rotate_right(sprite.rotation.rem_euclid(4) as usize);

            for (bitmap_corner, target_corner) in
                bitmap_corners.into_iter().zip(sprite.target_rect.corners())
            {
                let vertex = SpriteVertex {
                    position: target_corner.as_f32().to_array(),
                    bitmap_position: bitmap_corner.as_f32().to_array(),
                };
                vertices.push(vertex);
                aux_vertices.push(aux);
            }
        }

        self.vertex_array.bind(gl);
        self.array_buffer.buffer_data(gl, &vertices);
        self.aux_array_buffer.buffer_data(gl, &aux_vertices);
        self.element_buffer.buffer_data(gl, &indices);

        self.shader.use_program(gl);

        self.shader.uniform(gl, "device_from", &device_from);

        let uv_from_bitmap = GlTexture::gltexture_from_bitmap_with_size(bitmap_size);
        self.shader.uniform(gl, "uv_from_bitmap", &uv_from_bitmap);
    }

    pub unsafe fn draw(&mut self, gl: &glow::Context) {
        gl.draw_elements(
            glow::TRIANGLES,
            self.element_buffer.len() as i32,
            glow::UNSIGNED_INT,
            0,
        );
    }
}
