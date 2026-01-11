use crate::{
    math::{affine_map::AffineMap, point::Point},
    simulation::Particle,
    simulation_widgets::{draggable_icon_widget, labeled_drag_value},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Outflow {
    pub center: Point<f64>,
    pub radius: f64,
    pub triggered: bool,
    pub always_on: bool,
}

impl Default for Outflow {
    fn default() -> Self {
        Self {
            center: Point::ZERO,
            radius: 10.0,
            triggered: false,
            always_on: false,
        }
    }
}

impl Outflow {
    pub const ICON: egui::ImageSource<'static> = egui::include_image!("force_icons/vacuum.png");

    pub fn apply(&mut self, particles: &mut Vec<Particle>) {
        if self.always_on || self.triggered {
            particles.retain(|particle| {
                let r = particle.position.distance(self.center);
                r > self.radius
            });
        }
        self.triggered = false;
    }

    pub fn trigger(&mut self, _time: f64) {
        self.triggered = true;
    }

    pub fn settings_ui(&mut self, ui: &mut egui::Ui) {
        labeled_drag_value(ui, "Radius:", &mut self.radius, 1.0..=50.0, 0.5);
        ui.checkbox(&mut self.always_on, "Always On");
    }

    pub fn widget(
        &mut self,
        ui: &mut egui::Ui,
        sense: egui::Sense,
        selected: &mut bool,
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
