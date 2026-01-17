use crate::{
    forces::{ConservativeForce, Force},
    math::{affine_map::AffineMap, point::Point},
    piecewise_linear::PiecewiseLinear,
    simulation_widgets::{draggable_icon_widget, labeled_drag_value},
    utils::ReflectEnum,
    widgets::enum_choice_buttons,
};
use derive_more::From;
use egui::{ImageSource, Sense, Ui};
use serde::{Deserialize, Serialize};

pub trait RadialFunction {
    fn eval(&self, r: f64) -> f64;

    fn integrate(&self, r: f64) -> f64;

    fn settings_ui(&mut self, ui: &mut egui::Ui);
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GravityFunction {
    pub radius: f64,

    pub density: f64,
}

impl Default for GravityFunction {
    fn default() -> Self {
        Self {
            radius: 5.0,
            density: 80.0,
        }
    }
}

impl RadialFunction for GravityFunction {
    fn eval(&self, r: f64) -> f64 {
        if r < self.radius {
            self.density * r
        } else {
            self.density * (self.radius * self.radius * self.radius) / (r * r)
        }
    }

    fn integrate(&self, r: f64) -> f64 {
        // For r < radius: p = -1/2 * density * (3*radius^2 - r^2)
        // For r >= radius: p = -density * radius^3 / r

        if r < self.radius {
            -0.5 * self.density * (3.0 * self.radius * self.radius - r * r)
        } else {
            -self.density * self.radius * self.radius * self.radius / r
        }
    }

    fn settings_ui(&mut self, ui: &mut Ui) {
        labeled_drag_value(ui, "Radius:", &mut self.radius, 0.0..=40.0, 0.1);

        labeled_drag_value(ui, "Density:", &mut self.density, 1.0..=500.0, 0.5);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiecewiseLinearRadialFunction {
    /// Must be 0 at 0.
    function: PiecewiseLinear,
}

impl PiecewiseLinearRadialFunction {
    pub fn ring_attraction(strength: f64) -> Self {
        let knots = vec![
            crate::math::point::Point(20.0, 0.0),
            crate::math::point::Point(25.0, -strength),
            crate::math::point::Point(35.0, strength),
            crate::math::point::Point(40.0, 0.0),
        ];
        let function = PiecewiseLinear::new(knots);
        Self { function }
    }
}

impl Default for PiecewiseLinearRadialFunction {
    fn default() -> Self {
        Self::ring_attraction(50.0)
    }
}

impl PiecewiseLinearRadialFunction {
    pub fn new(function: PiecewiseLinear) -> Self {
        Self { function }
    }
}

impl RadialFunction for PiecewiseLinearRadialFunction {
    fn eval(&self, r: f64) -> f64 {
        self.function.eval(r)
    }

    fn integrate(&self, r: f64) -> f64 {
        self.function.integrate(r)
    }

    fn settings_ui(&mut self, ui: &mut Ui) {
        ui.label("Settings not implemented!");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub enum AnyRadialFunction {
    Gravity(GravityFunction),
    PiecewiseLinear(PiecewiseLinearRadialFunction),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RadialFunctionKind {
    Gravity,
    PiecewiseLinear,
}

impl RadialFunctionKind {
    pub const ALL: [Self; 2] = [Self::Gravity, Self::PiecewiseLinear];
}

impl ReflectEnum for RadialFunctionKind {
    fn all() -> &'static [Self] {
        &Self::ALL
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Gravity => "Gravity",
            Self::PiecewiseLinear => "PiecewiseLinear",
        }
    }
}

impl AnyRadialFunction {
    pub fn as_radial_function(&self) -> &dyn RadialFunction {
        match self {
            Self::Gravity(this) => this,
            Self::PiecewiseLinear(this) => this,
        }
    }

    pub fn as_radial_function_mut(&mut self) -> &mut dyn RadialFunction {
        match self {
            Self::Gravity(this) => this,
            Self::PiecewiseLinear(this) => this,
        }
    }

    pub fn kind(&self) -> RadialFunctionKind {
        match self {
            Self::Gravity(_) => RadialFunctionKind::Gravity,
            Self::PiecewiseLinear(_) => RadialFunctionKind::PiecewiseLinear,
        }
    }

    pub fn default_from_kind(kind: RadialFunctionKind) -> Self {
        match kind {
            RadialFunctionKind::Gravity => GravityFunction::default().into(),
            RadialFunctionKind::PiecewiseLinear => PiecewiseLinearRadialFunction::default().into(),
        }
    }
}

// pub enum RadialForce {
//     Gravity(Gravit)
// }

/// Force in radial direction, force in angular direction is zero
/// TODO: Compute potential from radial_force
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadialForce {
    pub center: Point<f64>,

    /// TODO: function(0) should be 0
    pub function: AnyRadialFunction,
}

impl RadialForce {
    pub const ICON: ImageSource<'static> = egui::include_image!("force_icons/gravity.png");
}

impl RadialForce {
    fn radial_plot_ui(ui: &mut egui::Ui, name: &str, mut function: impl FnMut(f64) -> f64) {
        // Plot function from r = 0 to r = 40
        let n_points = 128;

        let points: egui_plot::PlotPoints = (0..=n_points)
            .map(|i| {
                let r = (i as f64) * (40.0 / n_points as f64);
                let f = function(r);
                [r, f]
            })
            .collect();

        let line = egui_plot::Line::new("force", points);
        egui_plot::Plot::new(name)
            // .legend(egui_plot::Legend::default())
            .view_aspect(2.0)
            .height(100.0)
            .show_axes(false)
            .allow_zoom(false)
            .allow_drag(false)
            .show(ui, |plot_ui| {
                plot_ui.line(line);
            });
    }
}

impl Force for RadialForce {
    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        // Choose function kind
        let mut kind = self.function.kind();
        if enum_choice_buttons(ui, Some("Kind"), &mut kind) {
            self.function = AnyRadialFunction::default_from_kind(kind);
        }

        self.function.as_radial_function_mut().settings_ui(ui);

        let function = self.function.as_radial_function();

        ui.label("Force plot");
        Self::radial_plot_ui(ui, "force_plot", |r| function.eval(r));

        ui.label("Potential");
        Self::radial_plot_ui(ui, "potential_plot", |r| function.integrate(r));
    }

    fn force(&self, _simulation_time: f64, position: Point<f64>) -> Point<f64> {
        let delta = self.center - position;
        let r = delta.norm();
        // self.function is 0 at r=0 and continuous, so this is ok.
        if r < 1e-6 {
            Point::ZERO
        } else {
            let f = self.function.as_radial_function().eval(r);
            delta * f * (1.0 / r)
        }
    }

    fn widget(
        &mut self,
        ui: &mut Ui,
        sense: Sense,
        selected: &mut bool,
        _simulation_time: f64,
        egui_from_simulation: AffineMap<f64>,
    ) {
        draggable_icon_widget(
            ui,
            sense,
            Self::ICON,
            &mut self.center,
            selected,
            egui_from_simulation,
        );
    }
}

impl ConservativeForce for RadialForce {
    fn potential(&self, _simulation_time: f64, position: Point<f64>) -> f64 {
        let r = position.distance(self.center);
        self.function.as_radial_function().integrate(r)
    }
}
