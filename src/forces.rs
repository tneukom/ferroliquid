use crate::{math::point::Point, simulation::Particle, utils::ReflectEnum, widgets::enum_combo};
use num_traits::{Float, FloatConst};
use std::ops::RangeInclusive;

#[enum_delegate::register]
pub trait Force {
    fn field(&self, center: Point<f64>, p: Point<f64>, time: f64) -> Point<f64>;

    fn apply(&self, center: Point<f64>, particles: &mut [Particle], time: f64, dt: f64) {
        for particle in particles {
            let force = self.field(center, particle.position, time);
            particle.velocity = particle.velocity + dt * force;
        }
    }

    fn trigger(&mut self, _time: f64) {}

    fn image(&self) -> egui::ImageSource<'static>;

    fn settings_ui(&mut self, ui: &mut egui::Ui);
}

pub struct Gravity {
    pub mass_radius: f64,
    pub mass_density: f64,
}

impl Default for Gravity {
    fn default() -> Self {
        Self {
            mass_radius: 5.0,
            mass_density: 80.0,
        }
    }
}

impl Force for Gravity {
    fn field(&self, center: Point<f64>, p: Point<f64>, _time: f64) -> Point<f64> {
        let dir = center - p;
        let r = dir.norm();
        let s = r.max(self.mass_radius);
        let f = (self.mass_radius * self.mass_radius * self.mass_radius) / (s * s * s);
        // For r < mass_radius: f = 1 for r >= mass_radius: f = mass_radius^3 / r^3
        self.mass_density * f * dir
    }

    fn image(&self) -> egui::ImageSource<'static> {
        egui::include_image!("force_icons/gravity.png")
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        labeled_drag_value(ui, "Mass Radius:", &mut self.mass_radius, 1.0..=20.0, 0.1);
        labeled_drag_value(ui, "Mass Density:", &mut self.mass_density, 1.0..=40.0, 0.1);
    }
}

pub struct Swirl {
    pub force: f64,
    pub radius: f64,
}

impl Default for Swirl {
    fn default() -> Self {
        Self {
            force: 10.0,
            radius: 5.0,
        }
    }
}

impl Force for Swirl {
    fn field(&self, center: Point<f64>, p: Point<f64>, _time: f64) -> Point<f64> {
        let dir = center - p;

        // Constant speed
        let r = dir.norm();
        if r < 1.0 {
            Point::ZERO
        } else if r > self.radius {
            Point::ZERO
        } else {
            self.force * dir.perp_ccw() / r
        }

        // Speed proportional to r
        // if dir.norm() > self.radius {
        //     Point::ZERO
        // } else {
        //     let perp = dir.perp_ccw();
        //     self.speed * perp
        // }
    }

    fn image(&self) -> egui::ImageSource<'static> {
        egui::include_image!("force_icons/swirl.png")
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        labeled_drag_value(ui, "Force:", &mut self.force, -200.0..=200.0, 1.0);
        labeled_drag_value(ui, "Radius:", &mut self.radius, 1.0..=20.0, 0.1);
    }
}

pub struct UniformForce {
    pub angle: f64,
    pub strength: f64,
}

impl Default for UniformForce {
    fn default() -> Self {
        Self {
            angle: 90.0.to_radians(),
            strength: 80.0,
        }
    }
}

impl Force for UniformForce {
    fn field(&self, _center: Point<f64>, _p: Point<f64>, _time: f64) -> Point<f64> {
        self.strength * Point(self.angle.cos(), self.angle.sin())
    }

    fn image(&self) -> egui::ImageSource<'static> {
        egui::include_image!("force_icons/uniform.png")
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        labeled_angle_drag_value(ui, "Angle:", &mut self.angle);
        labeled_drag_value(ui, "Strength:", &mut self.strength, 0.0..=100.0, 0.5);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub struct Shockwave {
    /// Force is active in an annulus of the given radial width
    pub width: f64,

    pub start_time: f64,

    /// Speed in 1/s
    pub speed: f64,

    pub strength: f64,

    pub kind: ShockwaveKind,
}

impl Default for Shockwave {
    fn default() -> Self {
        Self {
            width: 5.0,
            start_time: 1e20,
            speed: 10.0,
            strength: 100.0,
            kind: ShockwaveKind::Constant,
        }
    }
}

impl Force for Shockwave {
    fn field(&self, center: Point<f64>, p: Point<f64>, time: f64) -> Point<f64> {
        let dir = p - center;
        let r = dir.norm();
        let s = r - (time - self.start_time) * self.speed;
        self.kind.wave(self.width, s) * self.strength * dir / r
    }

    fn trigger(&mut self, time: f64) {
        println!("Triggered at time {time}");
        self.start_time = time;
    }

    fn image(&self) -> egui::ImageSource<'static> {
        egui::include_image!("force_icons/shockwave.png")
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        enum_combo(ui, "Kind:", &mut self.kind);
        labeled_drag_value(ui, "Width:", &mut self.width, 1.0..=20.0, 0.5);
        labeled_drag_value(ui, "Speed:", &mut self.speed, 1.0..=100.0, 1.0);
        labeled_drag_value(ui, "Strength:", &mut self.strength, 1.0..=500.0, 1.0);
    }
}

fn labeled_drag_value(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f64,
    range: RangeInclusive<f64>,
    speed: f64,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(value).range(range).speed(speed));
    });
}

fn labeled_angle_drag_value(ui: &mut egui::Ui, label: &str, angle: &mut f64) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut angle_deg = angle.to_degrees();
        ui.add(
            egui::DragValue::new(&mut angle_deg)
                .range(-180.0..=180.0)
                .speed(0.5),
        );
        *angle = angle_deg.to_radians();
    });
}

#[enum_delegate::implement(Force)]
pub enum AnyForce {
    Gravity(Gravity),
    Swirl(Swirl),
    UniformForce(UniformForce),
    Shockwave(Shockwave),
}

pub struct PlacedForce {
    pub position: Point<f64>,
    pub force: AnyForce,
}

impl PlacedForce {
    pub fn new(force: impl Into<AnyForce>, position: Point<f64>) -> Self {
        Self {
            force: force.into(),
            position,
        }
    }
}
