use crate::{
    camera::Camera,
    coordinate_frame::CoordinateFrames,
    math::{point::Point, rect::Rect},
};

#[derive(Debug, Clone)]
pub struct ViewInput {
    pub frames: CoordinateFrames,

    pub view_mouse: Point<f64>,

    pub world_mouse: Point<f64>,

    pub left_mouse_down: bool,
}

impl ViewInput {
    pub const EMPTY: Self = Self {
        frames: CoordinateFrames {
            window_size: Point(640.0, 480.0),
            viewport: Rect::low_high(Point::ZERO, Point(640.0, 480.0)),
        },

        view_mouse: Point::ZERO,
        world_mouse: Point::ZERO,

        left_mouse_down: false,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditMode {
    /// Only move camera, zoom in/out, interact with running program
    Pointer,

    Brush,
}

impl EditMode {
    pub fn ui_label(self) -> &'static str {
        match self {
            Self::Pointer => "Pointer",
            Self::Brush => "Brush",
        }
    }

    pub const ALL: [EditMode; 2] = [Self::Pointer, Self::Brush];
}

#[derive(Debug, Clone)]
pub struct ViewSettings {
    edit_mode: EditMode,
}

impl Default for ViewSettings {
    fn default() -> Self {
        Self {
            edit_mode: EditMode::Pointer,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Brush {}

#[derive(Debug, Clone)]
pub struct Brushing {
    pub brush: Brush,
    pub world_mouse: Point<f64>,
}

#[derive(Debug, Clone)]
pub enum UiState {
    Brushing(Brushing),
    Idle,
}

impl UiState {
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }
}

pub struct View {
    pub camera: Camera,
    pub grid_size: Option<i64>,

    pub ui_state: UiState,
}

impl View {
    pub fn new() -> View {
        View {
            camera: Camera::default(),
            grid_size: None,
            ui_state: UiState::Idle,
        }
    }

    fn handle_brushing(
        &mut self,
        mut op: Brushing,
        input: &ViewInput,
        settings: &ViewSettings,
    ) -> UiState {
        // Because the brushing op is started even if left mouse is not down, we need to exit if
        // mode changes.
        let mode_exited = ![EditMode::Brush].contains(&settings.edit_mode);
        let stop = mode_exited || !input.left_mouse_down;
        if stop {
            return UiState::Idle;
        }

        if !input.left_mouse_down {
            return UiState::Idle;
        }

        // TODO: Change
        op.world_mouse = input.world_mouse;

        UiState::Brushing(op)
    }

    /// Transition from None state
    pub fn begin_action(&mut self, input: &ViewInput, settings: &mut ViewSettings) -> UiState {
        match settings.edit_mode {
            EditMode::Pointer => {}
            EditMode::Brush => {
                if input.left_mouse_down {
                    let op = Brushing {
                        world_mouse: input.world_mouse,
                        brush: Brush::default(),
                    };
                    return self.handle_brushing(op, input, settings);
                }
            }
        }

        UiState::Idle
    }

    pub fn nearest_grid_vertex(&self, world_point: Point<f64>) -> Point<f64> {
        world_point.round()
    }

    pub fn handle_input(&mut self, input: &mut ViewInput, settings: &mut ViewSettings) {
        let ui_state = self.ui_state.clone();
        self.ui_state = match ui_state {
            UiState::Brushing(op) => self.handle_brushing(op, input, settings),
            UiState::Idle => self.begin_action(input, settings),
        };
    }

    pub fn tile_containing(&self, world_point: Point<f64>) -> Point<i64> {
        world_point.floor().cwise_as()
    }
}
