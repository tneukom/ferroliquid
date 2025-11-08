use crate::{math::point::Point, simulation::Particle};
use egui::{ImageSource, Ui};
use num_traits::Float;

#[enum_delegate::register]
pub trait Force {
    fn field(&self, center: Point<f64>, p: Point<f64>) -> Point<f64>;

    fn apply(&self, center: Point<f64>, particles: &mut [Particle], dt: f64) {
        for particle in particles {
            let force = self.field(center, particle.position);
            particle.velocity = particle.velocity + dt * force;
        }
    }

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
    fn field(&self, center: Point<f64>, p: Point<f64>) -> Point<f64> {
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
        ui.horizontal(|ui| {
            ui.label("Mass Radius:");
            ui.add(
                egui::DragValue::new(&mut self.mass_radius)
                    .range(1.0..=20.0)
                    .speed(0.1),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Mass Density:");
            ui.add(
                egui::DragValue::new(&mut self.mass_density)
                    .range(1.0..=40.0)
                    .speed(0.1),
            );
        });
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
    fn field(&self, center: Point<f64>, p: Point<f64>) -> Point<f64> {
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
        ui.horizontal(|ui| {
            ui.label("Force:");
            ui.add(
                egui::DragValue::new(&mut self.force)
                    .range(-200.0..=200.0)
                    .speed(1.0),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Radius:");
            ui.add(
                egui::DragValue::new(&mut self.radius)
                    .range(1.0..=20.0)
                    .speed(0.1),
            );
        });
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
    fn field(&self, _center: Point<f64>, _p: Point<f64>) -> Point<f64> {
        self.strength * Point(self.angle.cos(), self.angle.sin())
    }

    fn image(&self) -> ImageSource<'static> {
        egui::include_image!("force_icons/uniform.png")
    }

    fn settings_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Angle:");
            let mut angle_deg = self.angle.to_degrees();
            ui.add(
                egui::DragValue::new(&mut angle_deg)
                    .range(-180.0..=180.0)
                    .speed(0.5),
            );
            self.angle = angle_deg.to_radians();
        });
        ui.horizontal(|ui| {
            ui.label("Strength:");
            ui.add(
                egui::DragValue::new(&mut self.strength)
                    .range(0.0..=100.0)
                    .speed(0.5),
            );
        });
    }
}

#[enum_delegate::implement(Force)]
pub enum AnyForce {
    Gravity(Gravity),
    Swirl(Swirl),
    UniformForce(UniformForce),
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
