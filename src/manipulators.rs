use crate::{
    math::{affine_map::AffineMap, point::Point},
    simulation::Particle,
    utils::ReflectEnum,
    widgets::enum_combo,
};
use num_traits::{Float, FloatConst};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

#[enum_delegate::register]
pub trait Manipulator {
    fn apply(
        &mut self,
        center: Point<f64>,
        particles: &mut Vec<Particle>,
        simulation_time: f64,
        dt: f64,
    );

    fn trigger(&mut self, _simulation_time: f64) {}

    fn image(&self) -> egui::ImageSource<'static>;

    fn settings_ui(&mut self, ui: &mut egui::Ui);
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

impl Manipulator for Gravity {
    fn apply(
        &mut self,
        center: Point<f64>,
        particles: &mut Vec<Particle>,
        _simulation_time: f64,
        dt: f64,
    ) {
        for particle in particles {
            let dir = center - particle.position;
            let r = dir.norm();
            let s = r.max(self.mass_radius);
            let f = (self.mass_radius * self.mass_radius * self.mass_radius) / (s * s * s);
            // For r < mass_radius: f = 1 for r >= mass_radius: f = mass_radius^3 / r^3
            particle.velocity += self.mass_density * f * dir * dt;
        }
    }

    fn image(&self) -> egui::ImageSource<'static> {
        egui::include_image!("force_icons/gravity.png")
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        labeled_drag_value(ui, "Mass Radius:", &mut self.mass_radius, 1.0..=20.0, 0.1);
        labeled_drag_value(ui, "Mass Density:", &mut self.mass_density, 1.0..=40.0, 0.1);
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

impl Manipulator for Swirl {
    fn apply(
        &mut self,
        center: Point<f64>,
        particles: &mut Vec<Particle>,
        _simulation_time: f64,
        dt: f64,
    ) {
        for particle in particles {
            let dir = center - particle.position;

            // Constant speed
            let r = dir.norm();
            let force = if r < 1.0 {
                Point::ZERO
            } else if r > self.radius {
                Point::ZERO
            } else {
                self.force * dir.perp_ccw() / r
            };

            particle.velocity += dt * force;

            // Speed proportional to r
            // if dir.norm() > self.radius {
            //     Point::ZERO
            // } else {
            //     let perp = dir.perp_ccw();
            //     self.speed * perp
            // }
        }
    }

    fn image(&self) -> egui::ImageSource<'static> {
        egui::include_image!("force_icons/swirl.png")
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        labeled_drag_value(ui, "Force:", &mut self.force, -200.0..=200.0, 1.0);
        labeled_drag_value(ui, "Radius:", &mut self.radius, 1.0..=20.0, 0.1);
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UniformForce {
    pub angle: f64,
    pub strength: f64,
}

impl Default for UniformForce {
    fn default() -> Self {
        Self {
            angle: 90.0.to_radians(),
            strength: 120.0,
        }
    }
}

impl Manipulator for UniformForce {
    fn apply(
        &mut self,
        _center: Point<f64>,
        particles: &mut Vec<Particle>,
        _simulation_time: f64,
        dt: f64,
    ) {
        for particle in particles {
            let force = self.strength * Point(self.angle.cos(), self.angle.sin());
            particle.velocity += dt * force;
        }
    }

    fn image(&self) -> egui::ImageSource<'static> {
        egui::include_image!("force_icons/uniform.png")
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        labeled_angle_drag_value(ui, "Angle:", &mut self.angle);
        labeled_drag_value(ui, "Strength:", &mut self.strength, 0.0..=200.0, 1.0);
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
    /// Force is active in an annulus of the given radial width
    pub width: f64,

    pub start_simulation_time: f64,

    /// Speed in 1/s
    pub speed: f64,

    pub strength: f64,

    pub kind: ShockwaveKind,
}

impl Default for Shockwave {
    fn default() -> Self {
        Self {
            width: 5.0,
            start_simulation_time: 1e20,
            speed: 10.0,
            strength: 100.0,
            kind: ShockwaveKind::Constant,
        }
    }
}

impl Manipulator for Shockwave {
    fn apply(
        &mut self,
        center: Point<f64>,
        particles: &mut Vec<Particle>,
        simulation_time: f64,
        dt: f64,
    ) {
        for particle in particles {
            let dir = particle.position - center;
            let r = dir.norm();
            let s = r - (simulation_time - self.start_simulation_time) * self.speed;
            let force = self.kind.wave(self.width, s) * self.strength * dir / r;
            particle.velocity += dt * force;
        }
    }

    fn trigger(&mut self, time: f64) {
        println!("Triggered at time {time}");
        self.start_simulation_time = time;
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vacuum {
    pub radius: f64,
    pub triggered: bool,
    pub always_on: bool,
}

impl Default for Vacuum {
    fn default() -> Self {
        Self {
            radius: 10.0,
            triggered: false,
            always_on: false,
        }
    }
}

impl Manipulator for Vacuum {
    fn apply(&mut self, center: Point<f64>, particles: &mut Vec<Particle>, _time: f64, _dt: f64) {
        if self.always_on || self.triggered {
            particles.retain(|particle| {
                let r = particle.position.distance(center);
                r > self.radius
            });
        }
        self.triggered = false;
    }

    fn trigger(&mut self, _time: f64) {
        self.triggered = true;
    }

    fn image(&self) -> egui::ImageSource<'static> {
        egui::include_image!("force_icons/shockwave.png")
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        labeled_drag_value(ui, "Radius:", &mut self.radius, 1.0..=20.0, 0.5);
        ui.checkbox(&mut self.always_on, "Always On");
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[enum_delegate::implement(Manipulator)]
pub enum AnyManipulator {
    Gravity(Gravity),
    Swirl(Swirl),
    UniformForce(UniformForce),
    Shockwave(Shockwave),
    Vacuum(Vacuum),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacedManipulator {
    pub position: Point<f64>,
    pub manipulator: AnyManipulator,
}

impl PlacedManipulator {
    pub fn new(force: impl Into<AnyManipulator>, position: Point<f64>) -> Self {
        Self {
            manipulator: force.into(),
            position,
        }
    }

    pub fn widget(
        &mut self,
        ui: &mut egui::Ui,
        sense: egui::Sense,
        selected: &mut bool,
        simulation_time: f64,
        egui_from_simulation: AffineMap<f64>,
    ) {
        let image_source = self.manipulator.image();
        let image = egui::Image::new(image_source).sense(sense);

        let egui_position: egui::Pos2 = (egui_from_simulation * self.position).into();
        let response = ui.put(
            egui::Rect::from_center_size(egui_position.into(), egui::vec2(64.0, 64.0)),
            image,
        );

        if response.dragged() {
            let simulation_from_egui = egui_from_simulation.inv();
            let egui_drag_delta: Point<f64> = response.drag_delta().into();
            let simulation_drag_delta = simulation_from_egui.linear * egui_drag_delta;
            self.position = self.position + simulation_drag_delta;
            *selected = true;
        }

        if response.clicked() {
            *selected = true;
            self.manipulator.trigger(simulation_time);
        }

        // Red circle around selected force
        if *selected {
            let stroke = egui::Stroke::new(2.0, egui::Color32::RED);
            ui.painter()
                .circle_stroke(response.rect.center(), 32.0, stroke);
        }
    }
}
