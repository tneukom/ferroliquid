use crate::{
    field::Field,
    grid::CellType,
    math::{point::Point, rect::Rect},
    sides::{Orientation, Side, SideField},
    simulation::Simulation,
};

const DRAW_SCALE: f64 = 40.0;

#[derive(Debug, Clone, Copy)]
pub struct SimulationDebugDrawSettings {
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
    density: bool,
    defined: bool,
}

impl Default for SimulationDebugDrawSettings {
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
            density: false,
            defined: false,
        }
    }
}

pub fn simulation_draw_settings_widget(
    ui: &mut egui::Ui,
    settings: &mut SimulationDebugDrawSettings,
) {
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
    ui.checkbox(&mut settings.density, "Density");
    ui.checkbox(&mut settings.defined, "Defined");
}

pub fn draw_side_field(
    painter: &egui::Painter,
    font: egui::FontId,
    rect: egui::Rect,
    field: &SideField<f64>,
) {
    for side in field.indices() {
        // Compute the world-space center of the side depending on orientation
        let world_pos = match side.orientation {
            Orientation::Vertical => side.index.as_f64() + Point(0.0, 0.5),
            Orientation::Horizontal => side.index.as_f64() + Point(0.5, 0.0),
        };

        let velocity = field[side];
        // let velocity = self.simulation.grid.sides.boundary_constant[side];
        let text = format!("{:.1}", velocity);

        painter.text(
            rect.left_top() + (DRAW_SCALE * world_pos).into(),
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
    ui_rect: egui::Rect,
    field_bounds: Rect<i64>,
    mut f: impl FnMut(Point<i64>) -> String,
) {
    for coord in field_bounds.iter_indices() {
        let world_pos = coord.as_f64() + Point(0.5, 0.5);
        let text = f(coord);

        painter.text(
            ui_rect.left_top() + (DRAW_SCALE * world_pos).into(),
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
    ui_rect: egui::Rect,
    field: &Field<f64>,
) {
    draw_cell_texts(painter, font, ui_rect, field.bounds(), |coord| {
        format!("{:.1}", field[coord])
    });
}

pub fn draw_grid(painter: &egui::Painter, rect: egui::Rect, bounds: Rect<i64>) {
    let stroke = egui::Stroke::new(0.5, egui::Color32::RED);

    // Vertical lines
    for x in bounds.left()..=bounds.right() {
        let start = DRAW_SCALE * Point(x, bounds.top()).as_f64();
        let stop = DRAW_SCALE * Point(x, bounds.bottom()).as_f64();
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
        let start = DRAW_SCALE * Point(bounds.left(), y).as_f64();
        let stop = DRAW_SCALE * Point(bounds.right(), y).as_f64();
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
    ui_rect: egui::Rect,
    settings: &SimulationDebugDrawSettings,
) {
    // ui_rect = ui_rect.translate(egui::Vec2::splat(10.0));

    let font = egui::FontId::new(9.0, egui::FontFamily::Monospace);

    if settings.grid {
        draw_grid(painter, ui_rect, simulation.grid.bounds);
    }

    if settings.cell_types {
        let field = &simulation.grid.cells_type;
        draw_cell_texts(painter, font.clone(), ui_rect, field.bounds(), |coord| {
            let text = match field[coord] {
                CellType::Solid => "S",
                CellType::Air => "A",
                CellType::Fluid => "F",
            };
            text.to_string()
        });
    }

    // Draw particles
    if settings.particles {
        let particle_radius = 2.0;

        for particle in &simulation.particles {
            let center: egui::Vec2 = (DRAW_SCALE * particle.position).into();

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
            ui_rect,
            &simulation.grid.sides.velocity_interpolated,
        );
    }

    if settings.velocity_div_free {
        draw_side_field(
            painter,
            font.clone(),
            ui_rect,
            &simulation.grid.sides.velocity_div_free,
        );
    }

    if settings.velocity_correction {
        draw_side_field(
            painter,
            font.clone(),
            ui_rect,
            &simulation.grid.sides.velocity_correction,
        );
    }

    if settings.pressure {
        draw_cell_field(
            painter,
            font.clone(),
            ui_rect,
            &simulation.grid.cells_pressure,
        );
    }

    if settings.density {
        draw_cell_field(
            painter,
            font.clone(),
            ui_rect,
            &simulation.grid.cells_density,
        )
    }

    if settings.divergence {
        draw_cell_texts(
            painter,
            font.clone(),
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
            ui_rect,
            &simulation.grid.sides.boundary_constant,
        );
    }

    if settings.boundary_linear {
        draw_side_field(
            painter,
            font.clone(),
            ui_rect,
            &simulation.grid.sides.boundary_linear,
        );
    }

    if settings.defined {
        draw_side_field(
            painter,
            font.clone(),
            ui_rect,
            &simulation.grid.sides.defined,
        );
    }
}

pub fn debug_simulation_scene_ui(
    ui: &mut egui::Ui,
    scene_rect: &mut egui::Rect,
    simulation: &Simulation,
    settings: &SimulationDebugDrawSettings,
) {
    egui::Scene::new()
        .zoom_range(0.25..=4.0)
        .show(ui, scene_rect, |ui| {
            let size = DRAW_SCALE * simulation.grid.bounds.size().as_f64();

            let (response, painter) = ui.allocate_painter(size.into(), egui::Sense::click());
            let rect = response.rect;
            draw_simulation(simulation, &painter, rect, settings);
        });
}

pub struct SimulationDebugWindow {
    show_window: bool,
    debug_scene_rect: egui::Rect,
    simulation_debug_draw_settings: SimulationDebugDrawSettings,
}

impl SimulationDebugWindow {
    pub fn new() -> Self {
        Self {
            show_window: false,
            debug_scene_rect: egui::Rect::ZERO,
            simulation_debug_draw_settings: SimulationDebugDrawSettings::default(),
        }
    }

    pub fn window_toggle(&mut self, ui: &mut egui::Ui, simulation: &Simulation) {
        // ui.toggle_value(&mut self.show_window, "Debug Window");

        if ui
            .add(egui::Button::new("Debug Window").selected(self.show_window))
            .clicked()
        {
            self.show_window = !self.show_window;
        }

        egui::Window::new("Debug Window")
            .open(&mut self.show_window)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                egui::SidePanel::left("debug_controls").show_inside(ui, |ui| {
                    simulation_draw_settings_widget(ui, &mut self.simulation_debug_draw_settings);
                });

                debug_simulation_scene_ui(
                    ui,
                    &mut self.debug_scene_rect,
                    simulation,
                    &self.simulation_debug_draw_settings,
                );

                // Doesn't work properly, see https://github.com/emilk/egui/issues/901
                // egui::CentralPanel::default().show_inside(ui, |ui| {
                //
                // });
            });
    }
}
