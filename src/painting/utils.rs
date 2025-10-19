use glow::HasContext;

pub unsafe fn check_error(gl: &glow::Context) {
    let error = gl.get_error();
    if error != glow::NO_ERROR {
        println!("GL error {error}");
    }
}

pub const RECT_TRIANGLE_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];
