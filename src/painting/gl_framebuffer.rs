use crate::painting::gl_texture::GlTexture;
use glow::HasContext;

pub struct GlFramebuffer {
    pub id: glow::Framebuffer,
    pub width: i64,
    pub height: i64,
}

impl GlFramebuffer {
    pub unsafe fn new(gl: &glow::Context, texture: &GlTexture) -> Self {
        let id = gl.create_framebuffer().unwrap();
        gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(id));

        gl.framebuffer_texture_2d(
            glow::DRAW_FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            glow::TEXTURE_2D,
            Some(texture.id),
            0,
        );

        let status = gl.check_framebuffer_status(glow::DRAW_FRAMEBUFFER);
        if status != glow::FRAMEBUFFER_COMPLETE {
            panic!(
                "Framebuffer incomplete: {}",
                Self::display_framebuffer_status(status)
            );
        }

        gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);

        Self {
            id,
            width: texture.width,
            height: texture.height,
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
}
