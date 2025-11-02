use crate::{
    field::RgbaField,
    math::{
        point::Point,
        rect::Rect,
        rgba8::{Rgb8, Rgba8},
    },
    utils::ReflectEnum,
};
use cached::proc_macro::cached;
use egui::{AtomExt, IntoAtoms};

const COLOR_BUTTON_MARGIN: f32 = 2.0;

fn rgba_field_egui_texture(ui: &mut egui::Ui, rgba_field: &RgbaField) -> egui::TextureHandle {
    let image = egui::ColorImage::from_rgba_unmultiplied(
        [rgba_field.width() as usize, rgba_field.height() as usize],
        rgba_field.as_raw(),
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
    egui::Button::new(atoms).corner_radius(4)
}

pub fn icon_button<'a>(icon: egui::ImageSource<'a>, size: f32) -> egui::Button<'a> {
    let icon_size = egui::Vec2::splat(size);
    styled_button(icon.atom_size(icon_size))
}

pub fn styled_space(ui: &mut egui::Ui) {
    ui.add_space(6.0);
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

pub fn choice_buttons<'a, T: Copy + Eq>(
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
