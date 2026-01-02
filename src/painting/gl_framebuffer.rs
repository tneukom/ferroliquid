use crate::{
    field::RgbaField,
    math::{point::Point, rect::Rect, rgba8::Rgba8},
    painting::gl_texture::GlTexture,
};
use glow::HasContext;

pub struct GlFramebuffer {
    pub id: glow::Framebuffer,
    pub width: i64,
    pub height: i64,
}

impl GlFramebuffer {
    pub unsafe fn new(gl: &glow::Context, width: i64, height: i64) -> Self {
        let id = gl.create_framebuffer().unwrap();

        Self { id, width, height }
    }

    pub unsafe fn viewport(&self, gl: &glow::Context) {
        gl.viewport(0, 0, self.width as i32, self.height as i32);
    }

    pub unsafe fn with_color_attachments(gl: &glow::Context, attachments: &[&GlTexture]) -> Self {
        let first_attachment = attachments.first().unwrap();
        let framebuffer = Self::new(gl, first_attachment.width, first_attachment.height);

        framebuffer.bind(gl);
        for (i, attachment) in attachments.iter().enumerate() {
            framebuffer.attach_color(gl, attachment, i as u32);
        }
        let draw_buffers: Vec<u32> = (0..attachments.len() as u32)
            .map(|i| glow::COLOR_ATTACHMENT0 + i)
            .collect();
        gl.draw_buffers(&draw_buffers);

        framebuffer.assert_complete(gl);
        framebuffer.unbind(gl);

        framebuffer
    }

    pub fn size(&self) -> Point<i64> {
        return Point(self.width, self.height);
    }

    /// Needs to be bound
    pub unsafe fn attach_color(&self, gl: &glow::Context, texture: &GlTexture, i_attachment: u32) {
        assert!(i_attachment < 32);
        assert_eq!(texture.size(), self.size());

        gl.framebuffer_texture_2d(
            glow::DRAW_FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0 + i_attachment,
            glow::TEXTURE_2D,
            Some(texture.id),
            0,
        );
    }

    /// Needs to be bound
    pub unsafe fn assert_complete(&self, gl: &glow::Context) {
        let status = gl.check_framebuffer_status(glow::DRAW_FRAMEBUFFER);
        if status != glow::FRAMEBUFFER_COMPLETE {
            panic!(
                "Framebuffer incomplete: {}",
                Self::display_framebuffer_status(status)
            );
        }
    }

    pub unsafe fn bind(&self, gl: &glow::Context) {
        gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(self.id));
    }

    pub unsafe fn unbind(&self, gl: &glow::Context) {
        gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);
    }

    pub fn display_framebuffer_status(status: u32) -> &'static str {
        match status {
            glow::FRAMEBUFFER_COMPLETE => "GL_FRAMEBUFFER_COMPLETE",
            glow::FRAMEBUFFER_UNDEFINED => "GL_FRAMEBUFFER_UNDEFINED",
            glow::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => "GL_FRAMEBUFFER_INCOMPLETE_ATTACHMENT",
            glow::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => {
                "GL_FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT"
            }
            glow::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => "GL_FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER",
            glow::FRAMEBUFFER_INCOMPLETE_READ_BUFFER => "GL_FRAMEBUFFER_INCOMPLETE_READ_BUFFER",
            glow::FRAMEBUFFER_UNSUPPORTED => "GL_FRAMEBUFFER_UNSUPPORTED",
            glow::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => "GL_FRAMEBUFFER_INCOMPLETE_MULTISAMPLE",
            glow::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS => "GL_FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS",
            _ => "Unknown framebuffer status",
        }
    }

    pub unsafe fn read_color_attachment0(&self, gl: &glow::Context) -> RgbaField {
        let bounds = Rect::low_size(Point::ZERO, Point(self.width, self.height));
        let mut image = RgbaField::filled(bounds, Rgba8::ZERO);

        gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(self.id));
        gl.read_buffer(glow::COLOR_ATTACHMENT0);

        gl.read_pixels(
            0,
            0,
            self.width as i32,
            self.height as i32,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelPackData::Slice(Some(image.as_u8_slice_mut())),
        );

        gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);

        image
    }
}
