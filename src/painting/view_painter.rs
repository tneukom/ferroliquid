use super::grid_painter::GridPainter;
use crate::view::{View, ViewInput};
use crate::{
    camera::Camera, coordinate_frame::CoordinateFrames, math::rect::Rect,
    painting::line_painter::LinePainter,
};

/// What is necessary to paint the view
pub struct DrawView {
    camera: Camera,
    frames: CoordinateFrames,
    time: f64,
    grid_size: Option<i64>,
}

impl DrawView {
    pub fn from_view(
        view: &mut View,
        view_input: &ViewInput,
        frames: CoordinateFrames,
        time: f64,
    ) -> Self {
        Self {
            camera: view.camera,
            grid_size: view.grid_size,
            frames,
            time,
        }
    }
}

pub struct ViewPainter {
    pub grid_painter: GridPainter,
    pub line_painter: LinePainter,
    pub i_frame: usize,
}

impl ViewPainter {
    pub unsafe fn new(gl: &glow::Context) -> ViewPainter {
        ViewPainter {
            grid_painter: GridPainter::new(gl),
            line_painter: LinePainter::new(gl),
            i_frame: 0,
        }
    }

    pub unsafe fn draw_selection_outline(
        &mut self,
        gl: &glow::Context,
        rect: Rect<i64>,
        camera: &Camera,
        frames: &CoordinateFrames,
        time: f64,
    ) {
        let device_from_world = frames.device_from_view() * camera.view_from_world();
        self.line_painter
            .draw_rect(gl, rect.cwise_as(), device_from_world, time);
    }

    pub unsafe fn draw_view(&mut self, gl: &glow::Context, draw: &DrawView) {
        // Grid in the background
        // if let Some(grid_size) = draw.grid_size {
        //     self.grid_painter.draw(
        //         gl,
        //         world_bounds,
        //         grid_size as f64,
        //         &draw.frames,
        //         &draw.camera,
        //     );
        // }

        self.i_frame += 1;
    }
}
