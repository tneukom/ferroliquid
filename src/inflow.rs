use crate::{
    math::{affine_map::AffineMap, parallelogram::Parallelogram, point::Point, rgba8::Rgba8},
    palettes::Palette,
    utils::ReflectEnum,
    widgets::{enum_combo, palette_popup},
};
use egui::epaint;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InflowPattern {
    Noise,
    VerticalStripes,
    HorizontalStripes,
    DiagonalStripes,
}

impl InflowPattern {
    pub const ALL: [Self; 4] = [
        Self::Noise,
        Self::VerticalStripes,
        Self::HorizontalStripes,
        Self::DiagonalStripes,
    ];
}

impl ReflectEnum for InflowPattern {
    fn all() -> &'static [Self] {
        &Self::ALL
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Noise => "Noise",
            Self::VerticalStripes => "Vertical Stripes",
            Self::HorizontalStripes => "Horizontal Stripes",
            Self::DiagonalStripes => "Diagonal Stripes",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Inflow {
    pub center: Point<f64>,
    /// Unit vector
    pub direction: Point<f64>,
    pub width: f64,
    pub speed: f64,

    pub color_a: Rgba8,
    pub color_b: Rgba8,
    pub pattern: InflowPattern,
    pub pattern_scale: f64,
}

impl Inflow {
    pub fn polygon_corners(&self) {}

    pub fn rect(&self) -> Parallelogram<f64> {
        let dt = 1.0 / 60.0;
        let length = (self.speed * dt).max(2.0);

        let parallelogram = Parallelogram::new(
            Point::ZERO,
            length * self.direction,
            self.direction.perp_ccw() * self.width,
        );
        parallelogram.translated(self.center - parallelogram.center())
    }

    pub fn velocity(&self) -> Point<f64> {
        self.speed * self.direction
    }

    pub fn settings_ui(&mut self, ui: &mut egui::Ui) {
        let palette = &Palette::palettes()[0];
        palette_popup(ui, palette, &mut self.color_a);
        palette_popup(ui, palette, &mut self.color_b);

        enum_combo(ui, "Pattern", &mut self.pattern);

        ui.add(
            egui::DragValue::new(&mut self.pattern_scale)
                .range(0.1..=2.0)
                .speed(0.01),
        );
    }

    pub fn handle(
        ui: &mut egui::Ui,
        key: egui::Id,
        egui_position: Point<f64>,
    ) -> Option<Point<f64>> {
        // Draw handle
        let circle = egui::Shape::circle_filled(egui_position.into(), 10.0, egui::Color32::RED);

        let response = ui.interact(
            circle.visual_bounding_rect(),
            key,
            egui::Sense::click_and_drag(),
        );
        ui.painter().add(circle);

        if response.dragged()
            && let Some(interact_pointer_pos) = response.interact_pointer_pos()
        {
            let egui_dragged_to: Point<f64> = interact_pointer_pos.into();
            Some(egui_dragged_to)
        } else {
            None
        }
    }

    fn pointed_rectangle_polygon(
        rect: Parallelogram<f64>,
        fill: impl Into<egui::Color32>,
        stroke: impl Into<epaint::PathStroke>,
    ) -> egui::Shape {
        // Middle of line BC moved a bit to make the shape pointed.
        let shape_tip = (rect.corner_b() + rect.corner_c()) * 0.5 + rect.u * 0.25;
        let corners = [
            rect.corner_a(),
            rect.corner_b(),
            shape_tip,
            rect.corner_c(),
            rect.corner_d(),
        ];

        let egui_corners: Vec<egui::Pos2> =
            corners.into_iter().map(|corner| corner.into()).collect();
        egui::Shape::convex_polygon(egui_corners, fill, stroke)
    }

    pub fn widget(
        &mut self,
        ui: &mut egui::Ui,
        sense: egui::Sense,
        selected: &mut bool,
        key: impl Hash + Copy,
        egui_from_simulation: AffineMap<f64>,
    ) {
        let simulation_from_egui = egui_from_simulation.inv();
        let egui_inflow_rect = egui_from_simulation * self.rect();

        // Shape
        let polygon = Self::pointed_rectangle_polygon(
            egui_inflow_rect,
            egui::Color32::BLACK,
            egui::Stroke::NONE,
        );
        let polygon_bounds = polygon.visual_bounding_rect();
        ui.painter().add(polygon);

        let response = ui.interact(polygon_bounds, egui::Id::new("inflow").with(key), sense);

        if response.dragged() {
            let egui_drag_delta: Point<f64> = response.drag_delta().into();
            let simulation_drag_delta = simulation_from_egui.linear * egui_drag_delta;
            self.center = self.center + simulation_drag_delta;
            *selected = true;
        }

        if response.clicked() {
            *selected = true;
        }

        if !*selected {
            return;
        }

        // Selection outline
        let polygon = Self::pointed_rectangle_polygon(
            egui_inflow_rect.padded(5.0),
            egui::Color32::TRANSPARENT,
            egui::Stroke::new(2.0, egui::Color32::RED),
        );
        ui.painter().add(polygon);

        // Speed/direction handle
        let speed_handle_center = self.center + self.direction * self.speed / 10.0;
        if let Some(egui_dragged_to) = Self::handle(
            ui,
            egui::Id::new("inflow_direction_handle").with(key),
            egui_from_simulation * speed_handle_center,
        ) {
            let dragged_to = simulation_from_egui * egui_dragged_to;
            let draw_delta = dragged_to - self.center;
            self.direction = draw_delta.normalized();
            // At most 2 cells per step at 60 fps
            self.speed = (draw_delta.norm() * 10.0).min(60.0 * 2.0);
        }

        // Width handle
        let width_handle_center = self.center + self.direction.perp_ccw() * 0.5 * self.width;
        if let Some(egui_dragged_to) = Self::handle(
            ui,
            egui::Id::new("inflow_width_handle").with(key),
            egui_from_simulation * width_handle_center,
        ) {
            let dragged_to = simulation_from_egui * egui_dragged_to;
            self.width = (dragged_to - self.center).norm();
        }
    }
}
