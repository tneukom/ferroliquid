use glow::HasContext;
use log::warn;

pub fn show_gl_error(err: u32) -> &'static str {
    match err {
        glow::NO_ERROR => "NO_ERROR",
        glow::INVALID_ENUM => "INVALID_ENUM",
        glow::INVALID_VALUE => "INVALID_VALUE",
        glow::INVALID_OPERATION => "INVALID_OPERATION",
        glow::INVALID_FRAMEBUFFER_OPERATION => "INVALID_FRAMEBUFFER_OPERATION",
        glow::OUT_OF_MEMORY => "OUT_OF_MEMORY",
        glow::STACK_UNDERFLOW => "STACK_UNDERFLOW",
        glow::STACK_OVERFLOW => "STACK_OVERFLOW",
        _ => "UNKNOWN_ERROR",
    }
}

pub unsafe fn check_gl_error(gl: &glow::Context) {
    let error = gl.get_error();
    if error != glow::NO_ERROR {
        let error_str = show_gl_error(error);
        warn!("GL error {error_str} ({error})");
    }
}

pub const RECT_TRIANGLE_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];
