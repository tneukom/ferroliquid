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
use itertools::Itertools;
use ordered_float::NotNan;
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

        ui.label("Force");
        radial_plot_ui(ui, "force_plot", |r| self.eval(r));

        ui.label("Potential");
        radial_plot_ui(ui, "potential_plot", |r| self.integrate(r));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PiecewiseLinearRadialFunction {
    strength: f64,

    /// Must be 0 at 0.
    function: PiecewiseLinear,
}

impl Default for PiecewiseLinearRadialFunction {
    fn default() -> Self {
        Self::ring_attraction(5.0)
    }
}

impl PiecewiseLinearRadialFunction {
    pub fn ring_attraction(strength: f64) -> Self {
        let knots = vec![
            crate::math::point::Point(20.0, 0.0),
            crate::math::point::Point(25.0, -25.0),
            crate::math::point::Point(35.0, 25.0),
            crate::math::point::Point(40.0, 0.0),
        ];
        let function = PiecewiseLinear::new(knots);
        Self { function, strength }
    }

    pub fn editable_plot(&mut self, ui: &mut egui::Ui) {
        let mut knots = self.function.knots().to_vec();

        let plot_points: Vec<egui_plot::PlotPoint> = knots
            .iter()
            .map(|knot| egui_plot::PlotPoint::new(knot.x, knot.y))
            .collect();

        egui_plot::Plot::new("piecewise_linear")
            // .legend(egui_plot::Legend::default())
            .view_aspect(2.0)
            .height(100.0)
            .show_axes(false)
            .allow_zoom(false)
            .allow_drag(false)
            .default_x_bounds(0.0, 50.0)
            .default_y_bounds(-25.0, 25.0)
            .show(ui, |plot_ui| {
                // Plot points
                plot_ui.points(
                    egui_plot::Points::new("points", egui_plot::PlotPoints::Borrowed(&plot_points))
                        .radius(5.0),
                );

                // Plot line
                plot_ui.line(egui_plot::Line::new(
                    "line",
                    egui_plot::PlotPoints::Borrowed(&plot_points),
                ));

                if plot_ui.response().dragged()
                    && let Some(egui_drag_stop) = plot_ui.pointer_coordinate()
                {
                    let egui_drag_delta = plot_ui.pointer_coordinate_drag_delta();
                    let egui_drag_start = egui_drag_stop.to_pos2() - egui_drag_delta;
                    println!("drag_delta: {egui_drag_delta}, drag_start: {egui_drag_start}");

                    let drag_start: Point<f64> = egui_drag_start.into();
                    if let Some(picked_knot) = knots
                        .iter_mut()
                        .min_by_key(|knot| NotNan::new(knot.distance(drag_start)).unwrap())
                    {
                        if picked_knot.distance(drag_start) < 20.0 {
                            *picked_knot = Point(egui_drag_stop.x, egui_drag_stop.y);
                            // Clip into range [0, 50] x [-25, 25]
                            *picked_knot = Point(
                                picked_knot.x.clamp(0.0, 50.0),
                                picked_knot.y.clamp(-25.0, 25.0),
                            );
                            knots.sort_by(|lhs, rhs| lhs.x.total_cmp(&rhs.x));
                            self.function = PiecewiseLinear::new(knots);
                        }
                    }
                }
            });
    }
}

impl PiecewiseLinearRadialFunction {
    pub fn new(function: PiecewiseLinear) -> Self {
        Self {
            function,
            strength: 5.0,
        }
    }
}

impl RadialFunction for PiecewiseLinearRadialFunction {
    fn eval(&self, r: f64) -> f64 {
        self.strength * self.function.eval(r)
    }

    fn integrate(&self, r: f64) -> f64 {
        self.strength * self.function.integrate(r)
    }

    fn settings_ui(&mut self, ui: &mut Ui) {
        labeled_drag_value(ui, "Strength:", &mut self.strength, 0.5..=100.0, 0.5);

        ui.label("Force");
        self.editable_plot(ui);

        ui.label("Potential");
        radial_plot_ui(ui, "potential_plot", |r| self.integrate(r));
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

impl Force for RadialForce {
    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        // Choose function kind
        let mut kind = self.function.kind();
        if enum_choice_buttons(ui, Some("Kind"), &mut kind) {
            self.function = AnyRadialFunction::default_from_kind(kind);
        }

        self.function.as_radial_function_mut().settings_ui(ui);
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
