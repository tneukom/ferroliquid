use crate::math::{affine_map::AffineMap, point::Point};
use std::ops::RangeInclusive;

pub fn labeled_drag_value(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f64,
    range: RangeInclusive<f64>,
    speed: f64,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::Slider::new(value, range).drag_value_speed(speed));
    });
}

pub fn labeled_angle_drag_value(ui: &mut egui::Ui, label: &str, angle: &mut f64) {
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

pub fn draggable_icon_widget(
    ui: &mut egui::Ui,
    sense: egui::Sense,
    icon: egui::ImageSource<'static>,
    position: &mut Point<f64>,
    selected: &mut bool,
    egui_from_simulation: AffineMap<f64>,
) -> egui::Response {
    let image = egui::Image::new(icon).sense(sense);

    let egui_position: egui::Pos2 = (egui_from_simulation * *position).into();
    let response = ui.put(
        egui::Rect::from_center_size(egui_position.into(), egui::vec2(64.0, 64.0)),
        image,
    );

    if response.dragged() {
        let simulation_from_egui = egui_from_simulation.inv();
        let egui_drag_delta: Point<f64> = response.drag_delta().into();
        let simulation_drag_delta = simulation_from_egui.linear * egui_drag_delta;
        *position = *position + simulation_drag_delta;
        *selected = true;
    }

    // if response.clicked() {
    //     *selected = true;
    //     self.manipulator
    //         .as_manipulator_mut()
    //         .trigger(simulation_time);
    // }

    // Red circle around selected force
    if *selected {
        let stroke = egui::Stroke::new(2.0, egui::Color32::RED);
        ui.painter()
            .circle_stroke(response.rect.center(), 32.0, stroke);
    }

    response
}
