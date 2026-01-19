use crate::{
    field::RgbaField,
    math::{point::Point, rect::Rect, rgba8::Rgba8},
    palettes::Palette,
    utils::ReflectEnum,
};
use cached::proc_macro::cached;
use egui::{AtomExt, IntoAtoms};
use itertools::Itertools;

const COLOR_BUTTON_MARGIN: f32 = 2.0;

fn rgba_field_egui_texture(ui: &mut egui::Ui, rgba_field: &RgbaField) -> egui::TextureHandle {
    let image = egui::ColorImage::from_rgba_unmultiplied(
        [rgba_field.width() as usize, rgba_field.height() as usize],
        rgba_field.as_u8_slice(),
    );

    ui.ctx()
        .load_texture("icon_texture", image, Default::default())
}

#[cached(key = "(Rgba8, i64)", convert = r#"{ (rgba, icon_size) }"#)]
fn rgb_icon(ui: &mut egui::Ui, rgba: Rgba8, icon_size: i64) -> egui::TextureHandle {
    let bounds = Rect::low_size(Point(0, 0), Point(icon_size, icon_size));
    let rgba_field = RgbaField::filled(bounds, rgba);
    rgba_field_egui_texture(ui, &rgba_field)
}

/// Warning: A new egui texture is created for each material, and is cached forever.
pub fn rgba_button(ui: &mut egui::Ui, rgba: Rgba8, selected: bool) -> egui::Response {
    let icon = rgb_icon(ui, rgba, 28);

    let sized_texture = egui::load::SizedTexture::from(&icon);
    let button = egui::widgets::ImageButton::new(sized_texture).selected(selected);
    ui.add(button)
}

pub fn styled_button<'a>(atoms: impl IntoAtoms<'a>) -> egui::Button<'a> {
    egui::Button::new(atoms).corner_radius(2)
}

pub fn icon_button<'a>(icon: egui::ImageSource<'a>, size: f32) -> egui::Button<'a> {
    let icon_size = egui::Vec2::splat(size);
    styled_button(icon.atom_size(icon_size))
}

pub fn styled_space(ui: &mut egui::Ui) {
    ui.add_space(12.0);
}

pub fn enum_combo<T: ReflectEnum + PartialEq + 'static>(
    ui: &mut egui::Ui,
    title: &str,
    current: &mut T,
) {
    egui::ComboBox::from_label(title)
        .selected_text(current.as_str())
        .width(150.0)
        .show_ui(ui, |ui| {
            for &candidate in T::all() {
                ui.selectable_value(current, candidate, candidate.as_str());
            }
        });
}

pub fn choice_buttons<'a, T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    title: Option<&str>,
    choices: impl IntoIterator<Item = (T, &'a str)>,
    selected: &mut T,
) -> bool {
    let mut clicked = false;

    // TODO: Would be nicer if this was left of the buttons, but vertical centering doesn't work
    //   properly because the layout doesn't know the height of the widgets following the labels.
    //   It works if the label is after the buttons.
    if let Some(title) = title {
        ui.label(title);
    }

    for (choice, label) in choices.into_iter() {
        if ui
            .add(styled_button(label).selected(choice == *selected))
            .clicked()
        {
            *selected = choice;
            clicked = true;
        }
    }

    clicked
}

pub fn enum_choice_buttons<T: Copy + ReflectEnum + Eq>(
    ui: &mut egui::Ui,
    title: Option<&str>,
    selected: &mut T,
) -> bool {
    let choices = T::all()
        .into_iter()
        .map(|&choice| (choice, choice.as_str()));
    choice_buttons(ui, title, choices, selected)
}

fn palette_btn_style(ui: &mut egui::Ui) {
    // 2 pixel padding and spacing
    ui.style_mut().spacing.button_padding = egui::Vec2::splat(4.0);
    ui.spacing_mut().item_spacing = egui::Vec2::splat(6.0);

    // Set padding color to same as panel background
    let padding_fill = ui.style_mut().visuals.panel_fill;
    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = padding_fill;
    ui.style_mut().visuals.widgets.active.weak_bg_fill = padding_fill;
    ui.style_mut().visuals.widgets.noninteractive.weak_bg_fill = padding_fill;
}

pub fn color_button(ui: &mut egui::Ui, rgba: Rgba8, selected: bool) -> egui::Response {
    let button = egui::Button::new(()).fill(rgba).selected(selected);
    ui.add_sized([32.0, 32.0], button)
}

pub fn palette_widget(ui: &mut egui::Ui, palette: &Palette, rgba: &mut Rgba8) -> bool {
    let mut color_set = false;

    ui.scope(|ui| {
        palette_btn_style(ui);

        // 4 colors per row
        for chunk in &palette.colors.iter().chunks(4) {
            ui.horizontal(|ui| {
                for &choice in chunk {
                    if color_button(ui, choice, choice == *rgba).clicked() {
                        *rgba = choice;
                        color_set = true;
                    }
                }
            });
        }
    });

    color_set
}

pub fn palette_popup(ui: &mut egui::Ui, palette: &Palette, rgba: &mut Rgba8) {
    let button = egui::Button::new(()).fill(*rgba);
    let response = ui.add_sized([28.0, 28.0], button);

    egui::Popup::menu(&response).show(|ui| {
        palette_widget(ui, palette, rgba);
    });
}
