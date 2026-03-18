use crate::{utils::ReflectEnum, widgets::enum_choice_buttons, world::Energy};
use derive_more::{From, with_trait::Display};
use enum_map::{Enum, EnumMap};
use std::{collections::VecDeque, sync::Mutex};
use web_time::Instant;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Enum)]
pub enum TimingSection {
    Integration,
    SolvePressure,
    PrepareGrid,
    UpdateFinalVelocity,
    InterpolateParticleVelocities,
    Step,
}

#[derive(Clone, Copy, From)]
pub enum Event {
    TimingEvent(TimingSection, f64),
    ParticleCount(usize),
    Energy(Energy),
    FrameDuration(f64),
}

pub struct Events {
    frame_begin: Option<Instant>,
    frames: VecDeque<Vec<Event>>,
}

impl Events {
    pub const fn new() -> Self {
        Self {
            frame_begin: None,
            frames: VecDeque::new(),
        }
    }
}

pub static TRACE_EVENTS: Mutex<Events> = Mutex::new(Events::new());

pub fn trace_event(event: Event) {
    if let Some(active_frame) = TRACE_EVENTS.lock().unwrap().frames.back_mut() {
        let event = event.into();
        active_frame.push(event);
    }
}

pub fn trace_begin_frame() {
    let mut trace_events = TRACE_EVENTS.lock().unwrap();

    // Measure time since last frame
    if let Some(frame_begin) = trace_events.frame_begin {
        if let Some(active_frame) = trace_events.frames.back_mut() {
            let total = frame_begin.elapsed().as_secs_f64();
            // println!("total: {total}");
            active_frame.push(Event::FrameDuration(total));
        }
    }

    trace_events.frame_begin = Some(Instant::now());

    // Add new frame
    trace_events.frames.push_back(Vec::new());
    while trace_events.frames.len() > 128 {
        trace_events.frames.pop_front();
    }
}

pub struct MeasureDuration {
    pub section: TimingSection,
    pub instant: Instant,
}

impl MeasureDuration {
    pub fn new(section: TimingSection) -> Self {
        Self {
            section,
            instant: Instant::now(),
        }
    }
}

impl Drop for MeasureDuration {
    fn drop(&mut self) {
        trace_event(Event::TimingEvent(
            self.section,
            self.instant.elapsed().as_secs_f64(),
        ));
    }
}

#[derive(Default)]
struct FrameProfile {
    durations: EnumMap<TimingSection, f64>,
    whole_frame_duration: f64,
    energy: Energy,
    particle_count: usize,
}

impl FrameProfile {
    pub fn remaining_duration(&self) -> f64 {
        let total: f64 = self.durations.values().sum();
        // total = step + prepare + solve + ...
        // remainder = step - prepare - solve - ... = 2 * step - total
        2.0 * self.durations[TimingSection::Step] - total
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum ProfileTab {
    #[default]
    Timing,
    Energy,
    ParticleCount,
}

impl ProfileTab {
    pub const ALL: [ProfileTab; 3] = [Self::Timing, Self::Energy, Self::ParticleCount];
}

impl ReflectEnum for ProfileTab {
    fn all() -> &'static [Self] {
        &Self::ALL
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Timing => "Timing",
            Self::Energy => "Energy",
            Self::ParticleCount => "Particle Count",
        }
    }
}

struct StackedBarChart {
    pub bar_spacing: f64,
    pub bar_width: f64,
    pub charts: Vec<egui_plot::BarChart>,
}

impl StackedBarChart {
    pub fn bars(&self, values: impl IntoIterator<Item = f64>) -> Vec<egui_plot::Bar> {
        values
            .into_iter()
            .enumerate()
            .map(|(i, value)| {
                egui_plot::Bar::new(i as f64 * self.bar_spacing, value).stroke(egui::Stroke::NONE)
            })
            .collect()
    }

    pub fn stack(&mut self, title: impl Into<String>, values: impl IntoIterator<Item = f64>) {
        let bars = self.bars(values);
        let mut bar_chart = egui_plot::BarChart::new(title, bars).width(self.bar_width);

        if let Some(previous_bar_chart) = self.charts.last() {
            bar_chart = bar_chart.stack_on(&[previous_bar_chart]);
        }

        self.charts.push(bar_chart);
    }
}

pub struct ProfilerWindow {
    show_window: bool,
    paused: bool,
    tab: ProfileTab,
    frame_profiles: Vec<FrameProfile>,
}

impl ProfilerWindow {
    pub fn new() -> Self {
        Self {
            show_window: false,
            paused: false,
            tab: ProfileTab::Timing,
            frame_profiles: Vec::new(),
        }
    }

    pub fn update_frame_profiles(&mut self) {
        // Collect profile events into FrameProfiles
        self.frame_profiles = {
            let events = TRACE_EVENTS.lock().unwrap();
            let mut frame_profiles = Vec::new();
            for frame_events in &events.frames {
                let mut frame_profile = FrameProfile::default();
                for event in frame_events {
                    match event {
                        &Event::TimingEvent(section, duration) => {
                            frame_profile.durations[section] = duration
                        }
                        &Event::FrameDuration(whole_frame_duration) => {
                            frame_profile.whole_frame_duration = whole_frame_duration;
                        }
                        &Event::Energy(energy) => frame_profile.energy = energy,
                        &Event::ParticleCount(particle_count) => {
                            frame_profile.particle_count = particle_count;
                        }
                    }
                }

                frame_profiles.push(frame_profile);
            }

            frame_profiles
        };

        // The last frame is not finished at this point, drop it.
        self.frame_profiles.pop();
    }

    fn frame_plot_points(
        &self,
        mut f: impl FnMut(&FrameProfile) -> f64,
    ) -> egui_plot::PlotPoints<'static> {
        let delta_x = 1.0;

        // Line chart for the whole frame duration
        let line_points: Vec<_> = self
            .frame_profiles
            .iter()
            .enumerate()
            .map(|(i, frame)| egui_plot::PlotPoint::new(i as f64 * delta_x, f(frame)))
            .collect();
        egui_plot::PlotPoints::Owned(line_points)
    }

    pub fn energy_plot_ui(&mut self, ui: &mut egui::Ui) {
        let kinetic = egui_plot::Line::new(
            "Kinetic Energy",
            self.frame_plot_points(|frame| frame.energy.kinetic),
        );

        let potential = egui_plot::Line::new(
            "Potential Energy",
            self.frame_plot_points(|frame| frame.energy.potential),
        );

        let total = egui_plot::Line::new(
            "Total Energy",
            self.frame_plot_points(|frame| frame.energy.total()),
        );

        egui_plot::Plot::new("energy")
            .legend(egui_plot::Legend::default())
            // .default_y_bounds(0.0, 0.2)
            .show(ui, |plot_ui| {
                plot_ui.line(kinetic);
                plot_ui.line(potential);
                plot_ui.line(total);
            });
    }

    pub fn particle_count_plot_ui(&mut self, ui: &mut egui::Ui) {
        let particle_count = egui_plot::Line::new(
            "Particle count",
            self.frame_plot_points(|frame| frame.particle_count as f64),
        );

        egui_plot::Plot::new("particles")
            .legend(egui_plot::Legend::default())
            .default_y_bounds(0.0, 20_000.0)
            .show(ui, |plot_ui| {
                plot_ui.line(particle_count);
            });
    }

    pub fn timing_plot_ui(&mut self, ui: &mut egui::Ui) {
        let delta_x = 1.0;

        let mut stacked_bar_chart = StackedBarChart {
            bar_width: 1.0,
            bar_spacing: 1.0,
            charts: Vec::new(),
        };

        stacked_bar_chart.stack(
            "Prepare Grid",
            self.frame_profiles
                .iter()
                .map(|frame| frame.durations[TimingSection::PrepareGrid]),
        );

        stacked_bar_chart.stack(
            "Solve Pressure",
            self.frame_profiles
                .iter()
                .map(|frame| frame.durations[TimingSection::SolvePressure]),
        );

        stacked_bar_chart.stack(
            "Interpolate Particle Velocities",
            self.frame_profiles
                .iter()
                .map(|frame| frame.durations[TimingSection::InterpolateParticleVelocities]),
        );

        stacked_bar_chart.stack(
            "Update Final Velocity",
            self.frame_profiles
                .iter()
                .map(|frame| frame.durations[TimingSection::UpdateFinalVelocity]),
        );

        stacked_bar_chart.stack(
            "Integration",
            self.frame_profiles
                .iter()
                .map(|frame| frame.durations[TimingSection::Integration]),
        );

        stacked_bar_chart.stack(
            "Remainder",
            self.frame_profiles
                .iter()
                .map(|frame| frame.remaining_duration()),
        );

        // Line chart for the whole frame duration
        let line_points: Vec<_> = self
            .frame_profiles
            .iter()
            .enumerate()
            .map(|(i, frame)| {
                egui_plot::PlotPoint::new(i as f64 * delta_x, frame.whole_frame_duration)
            })
            .collect();
        let line = egui_plot::Line::new("whole_frame", egui_plot::PlotPoints::Owned(line_points));

        egui_plot::Plot::new("profile")
            .legend(egui_plot::Legend::default())
            // .default_y_bounds(0.0, 0.2)
            .include_y(0.0)
            .include_y(0.015)
            .show(ui, |plot_ui| {
                for bar_char in stacked_bar_chart.charts {
                    plot_ui.bar_chart(bar_char);
                }
                plot_ui.line(line);
            });
    }

    pub fn window_toggle(&mut self, ui: &mut egui::Ui) {
        if ui
            .add(egui::Button::new("Profiler Window").selected(self.show_window))
            .clicked()
        {
            self.show_window = !self.show_window;
        }

        let mut show_window = self.show_window;
        egui::Window::new("Profiler Window")
            .open(&mut show_window)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.paused, "Paused");
                    if !self.paused {
                        self.update_frame_profiles();
                    }

                    enum_choice_buttons(ui, None, &mut self.tab);
                });

                match self.tab {
                    ProfileTab::Timing => self.timing_plot_ui(ui),
                    ProfileTab::Energy => self.energy_plot_ui(ui),
                    ProfileTab::ParticleCount => self.particle_count_plot_ui(ui),
                };
            });
        self.show_window = show_window;
    }
}
