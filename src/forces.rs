use crate::{
    math::{affine_map::AffineMap, point::Point},
    radial_force::RadialForce,
    simulation::Particle,
    simulation_widgets::{draggable_icon_widget, labeled_angle_drag_value, labeled_drag_value},
    utils::ReflectEnum,
    widgets::enum_combo,
};
use derive_more::From;
use egui::ImageSource;
use num_traits::{Float, FloatConst};
use serde::{Deserialize, Serialize};

pub trait Force {
    fn settings_ui(&mut self, ui: &mut egui::Ui);

    fn force(&self, simulation_time: f64, position: Point<f64>) -> Point<f64>;

    fn apply(&self, particles: &mut [Particle], simulation_time: f64, dt: f64) {
        for particle in particles {
            // Force evaluation at midpoint
            let f = self.force(
                simulation_time,
                0.5 * (particle.position + particle.previous_position),
            );

            // Force evaluation at current position
            // let f = self.force(simulation_time, particle.position);

            // Force evaluation at current and previous position
            // let f = 0.5
            //     * (self.force(simulation_time, particle.position)
            //         + self.force(simulation_time, particle.previous_position));

            particle.velocity += f * dt;
        }
    }

    fn widget(
        &mut self,
        ui: &mut egui::Ui,
        sense: egui::Sense,
        selected: &mut bool,
        simulation_time: f64,
        egui_from_simulation: AffineMap<f64>,
    );
}

pub trait ConservativeForce: Force {
    fn potential(&self, simulation_time: f64, position: Point<f64>) -> f64;

    fn sum(&self, simulation_time: f64, particles: &[Particle]) -> f64 {
        let mut total = 0.0;
        for particle in particles {
            total += self.potential(simulation_time, particle.position);
        }
        total
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Swirl {
    pub center: Point<f64>,
    pub force: f64,
    pub radius: f64,
}

impl Swirl {
    pub const ICON: egui::ImageSource<'static> = egui::include_image!("force_icons/swirl.png");
}

impl Default for Swirl {
    fn default() -> Self {
        Self {
            center: Point::ZERO,
            force: 10.0,
            radius: 5.0,
        }
    }
}

impl Force for Swirl {
    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        labeled_drag_value(ui, "Force:", &mut self.force, -200.0..=200.0, 1.0);
        labeled_drag_value(ui, "Radius:", &mut self.radius, 1.0..=20.0, 0.1);
    }

    fn force(&self, _simulation_time: f64, position: Point<f64>) -> Point<f64> {
        let dir = self.center - position;

        // Constant speed
        let r = dir.norm();
        if r < 1.0 {
            Point::ZERO
        } else if r > self.radius {
            Point::ZERO
        } else {
            self.force * dir.perp_ccw() / r
        }
    }

    fn widget(
        &mut self,
        ui: &mut egui::Ui,
        sense: egui::Sense,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UniformForce {
    pub center: Point<f64>,
    pub angle: f64,
    pub strength: f64,
}

impl UniformForce {
    pub const ICON: egui::ImageSource<'static> = egui::include_image!("force_icons/uniform.png");
}

impl Default for UniformForce {
    fn default() -> Self {
        Self {
            center: Point::ZERO,
            angle: 90.0.to_radians(),
            strength: 120.0,
        }
    }
}

impl Force for UniformForce {
    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        labeled_angle_drag_value(ui, "Angle:", &mut self.angle);
        labeled_drag_value(ui, "Strength:", &mut self.strength, 0.0..=500.0, 1.0);
    }

    fn force(&self, _simulation_time: f64, _position: Point<f64>) -> Point<f64> {
        self.strength * Point(self.angle.cos(), self.angle.sin())
    }

    fn widget(
        &mut self,
        ui: &mut egui::Ui,
        sense: egui::Sense,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShockwaveKind {
    Sin,
    Constant,
}

impl ShockwaveKind {
    pub const ALL: [Self; 2] = [Self::Sin, Self::Constant];

    pub fn wave(self, width: f64, r: f64) -> f64 {
        match self {
            ShockwaveKind::Sin => {
                // sin function with one period in [0, width]
                if (r > 0.0) && (r < width) {
                    let s = r / width * 2.0 * f64::PI();
                    -s.sin()
                } else {
                    0.0
                }
            }
            ShockwaveKind::Constant => {
                // indicator function for [0, width]
                if (r > 0.0) && (r < width) { 1.0 } else { 0.0 }
            }
        }
    }
}

impl ReflectEnum for ShockwaveKind {
    fn all() -> &'static [Self] {
        &Self::ALL
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Sin => "Sin",
            Self::Constant => "Constant",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Shockwave {
    pub center: Point<f64>,

    /// Force is active in an annulus of the given radial width
    pub width: f64,

    pub start_simulation_time: f64,

    /// Speed in 1/s
    pub speed: f64,

    pub strength: f64,

    pub kind: ShockwaveKind,
}

impl Shockwave {
    pub const ICON: ImageSource<'static> = egui::include_image!("force_icons/shockwave.png");

    fn trigger(&mut self, time: f64) {
        println!("Triggered at time {time}");
        self.start_simulation_time = time;
    }
}

impl Default for Shockwave {
    fn default() -> Self {
        Self {
            center: Point::ZERO,
            width: 5.0,
            start_simulation_time: 1e20,
            speed: 10.0,
            strength: 100.0,
            kind: ShockwaveKind::Constant,
        }
    }
}

impl Force for Shockwave {
    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        enum_combo(ui, "Kind:", &mut self.kind);

        labeled_drag_value(ui, "Width:", &mut self.width, 1.0..=20.0, 0.5);
        labeled_drag_value(ui, "Speed:", &mut self.speed, 1.0..=100.0, 1.0);
        labeled_drag_value(ui, "Strength:", &mut self.strength, 1.0..=500.0, 1.0);
    }

    fn force(&self, simulation_time: f64, position: Point<f64>) -> Point<f64> {
        let dir = position - self.center;
        let r = dir.norm();
        let s = r - (simulation_time - self.start_simulation_time) * self.speed;
        self.kind.wave(self.width, s) * self.strength * dir / r
    }

    fn widget(
        &mut self,
        ui: &mut egui::Ui,
        sense: egui::Sense,
        selected: &mut bool,
        simulation_time: f64,
        egui_from_simulation: AffineMap<f64>,
    ) {
        if draggable_icon_widget(
            ui,
            sense,
            Self::ICON,
            &mut self.center,
            selected,
            egui_from_simulation,
        )
        .clicked()
        {
            self.trigger(simulation_time);
        };
    }
}

#[derive(Debug, Clone, From, Serialize, Deserialize)]
pub enum AnyForce {
    Swirl(Swirl),
    UniformForce(UniformForce),
    Shockwave(Shockwave),
    RadialForce(RadialForce),
}

impl AnyForce {
    pub fn as_force(&self) -> &dyn Force {
        match self {
            Self::Swirl(this) => this,
            Self::UniformForce(this) => this,
            Self::Shockwave(this) => this,
            Self::RadialForce(this) => this,
        }
    }

    pub fn as_conservative_force(&self) -> Option<&dyn ConservativeForce> {
        match self {
            Self::RadialForce(this) => Some(this),
            _ => None,
        }
    }

    pub fn as_force_mut(&mut self) -> &mut dyn Force {
        match self {
            Self::Swirl(this) => this,
            Self::UniformForce(this) => this,
            Self::Shockwave(this) => this,
            Self::RadialForce(this) => this,
        }
    }
}
