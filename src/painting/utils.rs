use glow::HasContext;

pub unsafe fn check_error(gl: &glow::Context) {
    let error = gl.get_error();
    if error != glow::NO_ERROR {
        println!("GL error {error}");
    }
}
