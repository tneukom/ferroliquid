use crate::field::Field;
use crate::grid::CellType;
use crate::math::point::Point;
use crate::math::rect::Rect;
use crate::sides::{Side, SideField, SideOrientation};
use crate::simulation::Simulation;
use egui::util::undoer::Settings;
use std::borrow::Borrow;

#[derive(Debug, Clone, Copy)]
pub struct SimulationDrawSettings {
    particles: bool,
    particle_velocity_labels: bool,
    velocity_interpolated: bool,
    divergence: bool,
    velocity_div_free: bool,
    velocity_correction: bool,
    divergence_corrected: bool,
    pressure: bool,
    grid: bool,
    cell_types: bool,
    boundary_constant: bool,
    boundary_linear: bool,
}

impl Default for SimulationDrawSettings {
    fn default() -> Self {
        Self {
            particles: true,
            particle_velocity_labels: false,
            velocity_interpolated: false,
            divergence: false,
            velocity_div_free: false,
            velocity_correction: false,
            divergence_corrected: false,
            pressure: false,
            grid: true,
            cell_types: false,
            boundary_constant: false,
            boundary_linear: false,
        }
    }
}

pub fn simulation_draw_settings_widget(ui: &mut egui::Ui, settings: &mut SimulationDrawSettings) {
    ui.checkbox(&mut settings.particles, "Particles");
    ui.checkbox(
        &mut settings.particle_velocity_labels,
        "Particle Velocity Labels",
    );
    ui.checkbox(&mut settings.velocity_interpolated, "Velocity Interpolated");
    ui.checkbox(&mut settings.divergence, "Divergence");
    ui.checkbox(&mut settings.velocity_div_free, "Velocity Div Free");
    ui.checkbox(&mut settings.velocity_correction, "Velocity Correction");
    ui.checkbox(&mut settings.divergence_corrected, "Divergence Corrected");
    ui.checkbox(&mut settings.pressure, "Pressure");
    ui.checkbox(&mut settings.grid, "Grid");
    ui.checkbox(&mut settings.cell_types, "Cell Types");
    ui.checkbox(&mut settings.boundary_constant, "Boundary Constant");
    ui.checkbox(&mut settings.boundary_linear, "Boundary Linear");
}

pub fn draw_side_field(
    painter: &egui::Painter,
    font: egui::FontId,
    scale: f64,
    rect: egui::Rect,
    field: &SideField<f64>,
) {
    for side in field.indices() {
        // Compute the world-space center of the side depending on orientation
        let world_pos = match side.orientation {
            SideOrientation::Vertical => side.index.as_f64() + Point(0.0, 0.5),
            SideOrientation::Horizontal => side.index.as_f64() + Point(0.5, 0.0),
        };

        let velocity = field[side];
        // let velocity = self.simulation.grid.sides.boundary_constant[side];
        let text = format!("{:.1}", velocity);

        painter.text(
            rect.left_top() + (scale * world_pos).into(),
            egui::Align2::CENTER_CENTER,
            text,
            font.clone(),
            egui::Color32::from_rgb(0, 0, 0),
        );
    }
}

pub fn draw_cell_texts(
    painter: &egui::Painter,
    font: egui::FontId,
    scale: f64,
    ui_rect: egui::Rect,
    field_bounds: Rect<i64>,
    mut f: impl FnMut(Point<i64>) -> String,
) {
    for coord in field_bounds.iter_indices() {
        let world_pos = coord.as_f64() + Point(0.5, 0.5);
        let text = f(coord);

        painter.text(
            ui_rect.left_top() + (scale * world_pos).into(),
            egui::Align2::CENTER_CENTER,
            text,
            font.clone(),
            egui::Color32::from_rgb(0, 128, 0),
        );
    }
}

pub fn draw_cell_field(
    painter: &egui::Painter,
    font: egui::FontId,
    scale: f64,
    ui_rect: egui::Rect,
    field: &Field<f64>,
) {
    draw_cell_texts(painter, font, scale, ui_rect, field.bounds(), |coord| {
        format!("{:.1}", field[coord])
    });
}

pub fn draw_grid(painter: &egui::Painter, scale: f64, rect: egui::Rect, bounds: Rect<i64>) {
    let stroke = egui::Stroke::new(0.5, egui::Color32::RED);

    // Vertical lines
    for x in bounds.left()..=bounds.right() {
        let start = scale * Point(x, bounds.top()).as_f64();
        let stop = scale * Point(x, bounds.bottom()).as_f64();
        painter.line(
            vec![
                rect.left_top() + start.into(),
                rect.left_top() + stop.into(),
            ],
            stroke,
        );
    }

    // Horizontal lines
    for y in bounds.top()..=bounds.bottom() {
        let start = scale * Point(bounds.left(), y).as_f64();
        let stop = scale * Point(bounds.right(), y).as_f64();
        painter.line(
            vec![
                rect.left_top() + start.into(),
                rect.left_top() + stop.into(),
            ],
            stroke,
        );
    }
}

pub fn divergence(u: &SideField<f64>, coord: Point<i64>) -> f64 {
    -u[Side::left_side(coord)] + u[Side::right_side(coord)] - u[Side::top_side(coord)]
        + u[Side::bottom_side(coord)]
}

pub fn draw_simulation(
    simulation: &Simulation,
    painter: &egui::Painter,
    mut ui_rect: egui::Rect,
    settings: &SimulationDrawSettings,
) {
    ui_rect = ui_rect.translate(egui::Vec2::splat(10.0));

    let draw_scale = 10.0;
    let font = egui::FontId::new(9.0, egui::FontFamily::Monospace);

    if settings.grid {
        draw_grid(painter, draw_scale, ui_rect, simulation.grid.bounds);
    }

    if settings.cell_types {
        let field = &simulation.grid.cells_type;
        draw_cell_texts(
            painter,
            font.clone(),
            draw_scale,
            ui_rect,
            field.bounds(),
            |coord| {
                let text = match field[coord] {
                    CellType::Solid => "S",
                    CellType::Air => "A",
                    CellType::Fluid => "F",
                };
                text.to_string()
            },
        );
    }

    // Draw particles
    if settings.particles {
        let particle_radius = 2.0;

        for particle in &simulation.particles {
            let center: egui::Vec2 = (draw_scale * particle.position).into();

            painter.circle_filled(
                ui_rect.left_top() + center,
                particle_radius,
                egui::Color32::from_rgb(255, 0, 0),
            );

            if settings.particle_velocity_labels {
                // Draw velocity text next to the particle
                let text = format!("{:.1},{:.1}", particle.velocity.x, particle.velocity.y);
                painter.text(
                    ui_rect.left_top() + center + egui::vec2(-20.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    text,
                    font.clone(),
                    egui::Color32::from_rgb(0, 0, 0),
                );
            }
        }
    }

    if settings.velocity_interpolated {
        draw_side_field(
            painter,
            font.clone(),
            draw_scale,
            ui_rect,
            &simulation.grid.sides.velocity_interpolated,
        );
    }

    if settings.velocity_div_free {
        draw_side_field(
            painter,
            font.clone(),
            draw_scale,
            ui_rect,
            &simulation.grid.sides.velocity_div_free,
        );
    }

    if settings.velocity_correction {
        draw_side_field(
            painter,
            font.clone(),
            draw_scale,
            ui_rect,
            &simulation.grid.sides.velocity_correction,
        );
    }

    if settings.pressure {
        draw_cell_field(
            painter,
            font.clone(),
            draw_scale,
            ui_rect,
            &simulation.grid.cells_pressure,
        );
    }

    if settings.divergence {
        draw_cell_texts(
            painter,
            font.clone(),
            draw_scale,
            ui_rect,
            simulation.grid.cells_pressure.bounds().padded(-1),
            |coord| {
                let div = divergence(&simulation.grid.sides.velocity_interpolated, coord);
                format!("{div:.1}")
            },
        )
    }

    if settings.divergence_corrected {
        draw_cell_texts(
            painter,
            font.clone(),
            draw_scale,
            ui_rect,
            simulation.grid.cells_pressure.bounds().padded(-1),
            |coord| {
                let div = divergence(&simulation.grid.sides.velocity_div_free, coord);
                format!("{div:.1}")
            },
        )
    }

    if settings.boundary_constant {
        draw_side_field(
            painter,
            font.clone(),
            draw_scale,
            ui_rect,
            &simulation.grid.sides.boundary_constant,
        );
    }

    if settings.boundary_linear {
        draw_side_field(
            painter,
            font.clone(),
            draw_scale,
            ui_rect,
            &simulation.grid.sides.boundary_linear,
        );
    }
}
