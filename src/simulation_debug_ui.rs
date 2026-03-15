use crate::{
    field::{Field, RgbaField},
    grid::CellType,
    math::{point::Point, rect::Rect, rgba8::Rgba8},
    sides::{Orientation, Side, SideField},
    world::World,
};
use ordered_float::NotNan;

const DRAW_SCALE: f64 = 40.0;

#[derive(Debug, Clone, Copy)]
pub struct SimulationDebugDrawSettings {
    particles: bool,
    previous_particles: bool,
    particle_velocity_labels: bool,
    velocity_interpolated: bool,
    divergence: bool,
    velocity_div_free: bool,
    velocity_correction: bool,
    velocity_boundary_corrected: bool,
    passable: bool,
    divergence_corrected: bool,
    pressure: bool,
    grid: bool,
    cell_types: bool,
    density: bool,
    side_weight: bool,
    defined: bool,
    distance: bool,
    smoothed_distance: bool,
    distance_grad: bool,
    distance_heaviside_step: bool,
    solid: bool,
}

impl Default for SimulationDebugDrawSettings {
    fn default() -> Self {
        Self {
            particles: true,
            previous_particles: false,
            particle_velocity_labels: false,
            velocity_interpolated: false,
            divergence: false,
            velocity_div_free: false,
            velocity_correction: false,
            velocity_boundary_corrected: false,
            passable: false,
            divergence_corrected: false,
            pressure: false,
            grid: true,
            cell_types: false,
            density: false,
            side_weight: false,
            defined: false,
            distance: false,
            smoothed_distance: false,
            distance_grad: false,
            distance_heaviside_step: false,
            solid: false,
        }
    }
}

pub fn simulation_draw_settings_widget(
    ui: &mut egui::Ui,
    settings: &mut SimulationDebugDrawSettings,
) {
    ui.checkbox(&mut settings.particles, "Particles");
    ui.checkbox(&mut settings.previous_particles, "Previous Particles");
    ui.checkbox(
        &mut settings.particle_velocity_labels,
        "Particle Velocity Labels",
    );
    ui.checkbox(&mut settings.velocity_interpolated, "Velocity Interpolated");
    ui.checkbox(&mut settings.divergence, "Divergence");
    ui.checkbox(&mut settings.velocity_div_free, "Velocity Div Free");
    ui.checkbox(&mut settings.velocity_correction, "Velocity Correction");
    ui.checkbox(
        &mut settings.velocity_boundary_corrected,
        "Velocity Boundary Corrected",
    );
    ui.checkbox(&mut settings.passable, "Passable");
    ui.checkbox(&mut settings.divergence_corrected, "Divergence Corrected");
    ui.checkbox(&mut settings.pressure, "Pressure");
    ui.checkbox(&mut settings.grid, "Grid");
    ui.checkbox(&mut settings.cell_types, "Cell Types");
    ui.checkbox(&mut settings.density, "Density");
    ui.checkbox(&mut settings.side_weight, "Side Weight");
    ui.checkbox(&mut settings.defined, "Defined");
    ui.checkbox(&mut settings.distance, "Distance");
    ui.checkbox(&mut settings.smoothed_distance, "Smoothed Distance");
    ui.checkbox(&mut settings.distance_grad, "Distance Grad");
    ui.checkbox(
        &mut settings.distance_heaviside_step,
        "Heaviside Step Distance",
    );
    ui.checkbox(&mut settings.solid, "Solid");
}

pub fn draw_side_field<T>(
    painter: &egui::Painter,
    font: egui::FontId,
    field: &SideField<T>,
    mut show: impl FnMut(&T) -> String,
) {
    for side in field.indices() {
        // Compute the world-space center of the side depending on orientation
        let world_pos = match side.orientation {
            Orientation::Vertical => side.index.as_f64() + Point(0.0, 0.5),
            Orientation::Horizontal => side.index.as_f64() + Point(0.5, 0.0),
        };

        painter.text(
            (DRAW_SCALE * world_pos).into(),
            egui::Align2::CENTER_CENTER,
            show(&field[side]),
            font.clone(),
            egui::Color32::from_rgb(0, 0, 0),
        );
    }
}

pub fn draw_side_float_field(painter: &egui::Painter, font: egui::FontId, field: &SideField<f64>) {
    draw_side_field(painter, font, field, |value| format!("{:.1}", value))
}

/// Iterate over grid points in bounds, starting at bounds.low() with the given spacing.
pub fn grid_nodes(bounds: Rect<f64>, spacing: f64) -> impl Iterator<Item = Point<f64>> + Clone {
    let indices_size = (bounds.size() / spacing).floor().as_i64();
    Rect::low_size(Point::ZERO, indices_size)
        .iter_closed()
        .map(move |index| bounds.low() + index.as_f64() * spacing)
}

/// Draws a square for each field index. The square is red if f(index) is negative, blue if
/// positive. The size of the square is proportional to |f(index)|.
pub fn draw_square_field(
    painter: &egui::Painter,
    grid_spacing: f64,
    bounds: Rect<f64>,
    mut f: impl FnMut(Point<f64>) -> f64,
) {
    let max_abs = grid_nodes(bounds, grid_spacing)
        .filter_map(|position| NotNan::new(f(position).abs()).ok())
        .max()
        .unwrap()
        .into_inner();

    for grid_node in grid_nodes(bounds, grid_spacing) {
        let value = f(grid_node);
        let center: egui::Pos2 = (DRAW_SCALE * grid_node).into();
        let size = (DRAW_SCALE * grid_spacing * value.abs() / max_abs) as f32;

        let color = if value < 0.0 {
            egui::Color32::from_rgba_premultiplied(255, 0, 0, 128)
        } else {
            egui::Color32::from_rgba_premultiplied(0, 0, 255, 128)
        };

        let rect = egui::Rect::from_center_size(center, egui::Vec2::splat(size));
        painter.rect_filled(rect, 0.0, color);
    }
}

pub fn draw_vector_field(
    painter: &egui::Painter,
    grid_spacing: f64,
    bounds: Rect<f64>,
    mut f: impl FnMut(Point<f64>) -> Point<f64>,
) {
    // let max_norm_squared = grid_nodes(bounds, grid_spacing)
    //     .filter_map(|position| NotNan::new(f(position).norm_squared()).ok())
    //     .max()
    //     .unwrap()
    //     .into_inner();

    for grid_node in grid_nodes(bounds, grid_spacing) {
        let u = f(grid_node);
        if !u.is_finite() {
            continue;
        }

        // let u = 2.0 * u / max_norm_squared;
        let u = 0.5 * u;
        let start = (DRAW_SCALE * grid_node).into();
        let stop = (DRAW_SCALE * (grid_node + u)).into();

        let stroke = egui::Stroke::new(0.5, egui::Color32::RED);
        painter.line(vec![start, stop], stroke);
    }
}

pub fn draw_image(painter: &egui::Painter, bounds: Rect<f64>, image: &RgbaField) {
    let egui_image = egui::ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        image.as_u8_slice(),
    );
    let texture =
        painter
            .ctx()
            .load_texture("highres_field", egui_image, egui::TextureOptions::NEAREST);

    painter.image(
        texture.id(),
        (bounds * DRAW_SCALE).into(),
        Rect::UNIT.into(),
        egui::Color32::WHITE,
    );
}

pub fn draw_highres_field_step(
    painter: &egui::Painter,
    grid_spacing: f64,
    bounds: Rect<f64>,
    mut f: impl FnMut(Point<f64>) -> f64,
) {
    // TODO: Some kind of CellGrid class (nodes in cells)
    let indices_size = (bounds.size() / grid_spacing).floor().as_i64();
    let image_bounds = Rect::low_size(Point::ZERO, indices_size + Point(1, 1));

    let image = RgbaField::from_map(image_bounds, |index| {
        let grid_node = bounds.low() + index.as_f64() * grid_spacing;
        let value = f(grid_node);
        if value < 0.0 {
            Rgba8::new(255, 0, 0, 128)
        } else {
            Rgba8::new(0, 0, 255, 128)
        }
    });

    let image_world_bounds = Rect::low_size(bounds.low(), indices_size.as_f64() * grid_spacing);
    draw_image(painter, image_world_bounds, &image);
}

pub fn draw_cell_texts(
    painter: &egui::Painter,
    font: egui::FontId,
    field_bounds: Rect<i64>,
    mut f: impl FnMut(Point<i64>) -> String,
) {
    for coord in field_bounds.iter_indices() {
        let world_pos = coord.as_f64() + Point(0.5, 0.5);
        let text = f(coord);

        painter.text(
            (DRAW_SCALE * world_pos).into(),
            egui::Align2::CENTER_CENTER,
            text,
            font.clone(),
            egui::Color32::from_rgb(0, 128, 0),
        );
    }
}

pub fn draw_cell_field(painter: &egui::Painter, font: egui::FontId, field: &Field<f64>) {
    draw_cell_texts(painter, font, field.bounds(), |coord| {
        format!("{:.1}", field[coord])
    });
}

pub fn draw_grid(painter: &egui::Painter, bounds: Rect<i64>) {
    let stroke = egui::Stroke::new(0.5, egui::Color32::RED);

    // Vertical lines
    for x in bounds.left()..=bounds.right() {
        let start = DRAW_SCALE * Point(x, bounds.top()).as_f64();
        let stop = DRAW_SCALE * Point(x, bounds.bottom()).as_f64();
        painter.line(vec![start.into(), stop.into()], stroke);
    }

    // Horizontal lines
    for y in bounds.top()..=bounds.bottom() {
        let start = DRAW_SCALE * Point(bounds.left(), y).as_f64();
        let stop = DRAW_SCALE * Point(bounds.right(), y).as_f64();
        painter.line(vec![start.into(), stop.into()], stroke);
    }
}

pub fn draw_simulation(
    world: &World,
    painter: &egui::Painter,
    settings: &SimulationDebugDrawSettings,
) {
    // ui_rect = ui_rect.translate(egui::Vec2::splat(10.0));
    let simulation = &world.simulation;
    let sides = &simulation.grid.sides;

    let font = egui::FontId::new(9.0, egui::FontFamily::Monospace);

    if settings.grid {
        draw_grid(painter, simulation.grid.bounds);
    }

    if settings.cell_types {
        let field = &simulation.grid.cells_type;
        draw_cell_texts(painter, font.clone(), field.bounds(), |coord| {
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
            let center: egui::Pos2 = (DRAW_SCALE * particle.position).into();

            painter.circle_filled(center, particle_radius, egui::Color32::from_rgb(255, 0, 0));

            if settings.particle_velocity_labels {
                // Draw velocity text next to the particle
                let text = format!("{:.1},{:.1}", particle.velocity.x, particle.velocity.y);
                painter.text(
                    center + egui::vec2(-20.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    text,
                    font.clone(),
                    egui::Color32::from_rgb(0, 0, 0),
                );
            }
        }
    }

    if settings.velocity_interpolated {
        draw_side_float_field(painter, font.clone(), &sides.velocity_interpolated);
    }

    if settings.passable {
        draw_side_float_field(painter, font.clone(), &sides.passable);
    }

    if settings.velocity_div_free {
        draw_side_float_field(painter, font.clone(), &sides.velocity_div_free);
    }

    if settings.velocity_correction {
        draw_side_float_field(painter, font.clone(), &sides.velocity_correction);
    }

    if settings.pressure {
        draw_cell_field(painter, font.clone(), &simulation.grid.cells_pressure);
    }

    if settings.density {
        draw_cell_field(painter, font.clone(), &simulation.grid.cells_density)
    }

    if settings.side_weight {
        draw_side_float_field(painter, font.clone(), &sides.weight)
    }

    if settings.divergence {
        draw_cell_texts(
            painter,
            font.clone(),
            simulation.grid.cells_pressure.bounds().padded(-1),
            |coord| {
                let div = sides.divergence(&sides.velocity_interpolated, coord);
                format!("{div:.1}")
            },
        )
    }

    if settings.divergence_corrected {
        draw_cell_texts(
            painter,
            font.clone(),
            simulation.grid.cells_pressure.bounds().padded(-1),
            |coord| {
                let div = sides.divergence(&sides.velocity_div_free, coord);
                format!("{div:.1}")
            },
        )
    }

    if settings.defined {
        draw_side_field(painter, font.clone(), &sides.defined, |&defined| {
            if defined {
                "Y".to_string()
            } else {
                "N".to_string()
            }
        });
    }

    if settings.distance {
        draw_cell_texts(
            painter,
            font.clone(),
            world.simulation.grid.bounds,
            |index| {
                let distance = simulation
                    .solid_boundary
                    .distance_at(index.as_f64() + Point(0.5, 0.5));
                match distance {
                    f64::INFINITY => "∞".to_string(),
                    f64::NEG_INFINITY => "-∞".to_string(),
                    distance => format!("{distance:.1}"),
                }
            },
        );
    }

    if settings.smoothed_distance {
        // draw_cell_field(painter, font, &solid.signed_distance);

        draw_square_field(
            painter,
            0.5,
            world.simulation.grid.bounds.as_f64(),
            |position| simulation.solid_boundary.smoothed_distance_at(position),
        );

        draw_cell_texts(
            painter,
            font.clone(),
            world.simulation.grid.bounds,
            |index| {
                let distance = simulation
                    .solid_boundary
                    .smoothed_distance_at(index.as_f64() + Point(0.5, 0.5));
                match distance {
                    f64::INFINITY => "∞".to_string(),
                    f64::NEG_INFINITY => "-∞".to_string(),
                    distance => format!("{distance:.1}"),
                }
            },
        );
    }

    if settings.distance_grad {
        draw_vector_field(
            painter,
            0.5,
            world.simulation.grid.bounds.as_f64(),
            |position| simulation.solid_boundary.grad_at(position),
        );
    }

    if settings.distance_heaviside_step {
        draw_highres_field_step(
            painter,
            0.125,
            world.simulation.grid.bounds.as_f64(),
            |position| simulation.solid_boundary.smoothed_distance_at(position),
        );
    }

    if settings.velocity_boundary_corrected {
        draw_vector_field(
            painter,
            0.25,
            world.simulation.grid.bounds.padded(-1).as_f64(),
            |position| 0.025 * simulation.velocity_boundary_corrected(position),
        )
    }

    if settings.solid {
        // TODO: There's a slight offset between solid_image and distance_heaviside_step.
        draw_image(painter, world.bounds().as_f64(), &world.solid);
    }
}

pub fn debug_simulation_scene_ui(
    ui: &mut egui::Ui,
    scene_rect: &mut egui::Rect,
    world: &World,
    settings: &SimulationDebugDrawSettings,
) {
    egui::Scene::new()
        .zoom_range(0.25..=4.0)
        .show(ui, scene_rect, |ui| {
            let size = DRAW_SCALE * world.simulation.grid.bounds.size().as_f64();

            let (_response, painter) = ui.allocate_painter(size.into(), egui::Sense::click());

            draw_simulation(world, &painter, settings);
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

    pub fn window_toggle(&mut self, ui: &mut egui::Ui, world: &World) {
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
                    world,
                    &self.simulation_debug_draw_settings,
                );

                // Doesn't work properly, see https://github.com/emilk/egui/issues/901
                // egui::CentralPanel::default().show_inside(ui, |ui| {
                //
                // });
            });
    }
}
