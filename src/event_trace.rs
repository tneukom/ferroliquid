use derive_more::From;
use std::{collections::VecDeque, sync::Mutex};
use web_time::Instant;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TimingSection {
    Total,
    Integration,
    SolvePressure,
    Step,
}

#[derive(Clone, Copy, From)]
pub enum Event {
    TimingEvent(TimingSection, f64),
}

pub struct Events {
    frames: VecDeque<Vec<Event>>,
}

impl Events {
    pub const fn new() -> Self {
        Self {
            frames: VecDeque::new(),
        }
    }
}

pub static TRACE_EVENTS: Mutex<Events> = Mutex::new(Events::new());

pub fn trace_event(event: impl Into<Event>) {
    if let Some(active_frame) = TRACE_EVENTS.lock().unwrap().frames.back_mut() {
        let event = event.into();
        active_frame.push(event);
    }
}

pub fn trace_begin_frame() {
    let mut trace_events = TRACE_EVENTS.lock().unwrap();
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
struct FrameDurations {
    total: f64,
    integration: f64,
    solve_pressure: f64,
    step: f64,
}

/// Duration plot for a single section, multiple of these are stacked
fn duration_plot(name: &str, durations: impl IntoIterator<Item = f64>) -> egui_plot::BarChart {
    let bars: Vec<egui_plot::Bar> = durations
        .into_iter()
        .enumerate()
        .map(|(i, duration)| egui_plot::Bar::new(i as f64, duration))
        .collect();
    egui_plot::BarChart::new(name, bars)
}

pub fn events_ui(ui: &mut egui::Ui) {
    let frames = {
        let events = TRACE_EVENTS.lock().unwrap();
        let mut frames = Vec::new();
        for frame_events in &events.frames {
            let mut durations = FrameDurations::default();
            for event in frame_events {
                match event {
                    &Event::TimingEvent(TimingSection::Step, duration) => durations.step = duration,
                    &Event::TimingEvent(TimingSection::Integration, duration) => {
                        durations.integration = duration
                    }
                    &Event::TimingEvent(TimingSection::SolvePressure, duration) => {
                        durations.solve_pressure = duration
                    }
                    _ => {}
                }
            }

            frames.push(durations);
        }

        frames
    };

    egui_plot::Plot::new("profile")
        // .legend(egui_plot::Legend::default())
        .show(ui, |plot_ui| {
            let solve_pressure_plot =
                duration_plot("solve", frames.iter().map(|frame| frame.solve_pressure));

            let integration_plot =
                duration_plot("integration", frames.iter().map(|frame| frame.integration))
                    .stack_on(&[&solve_pressure_plot]);

            plot_ui.bar_chart(solve_pressure_plot);
            plot_ui.bar_chart(integration_plot);
        });
}

pub struct ProfilerWindow {
    show_window: bool,
}

impl ProfilerWindow {
    pub fn new() -> Self {
        Self { show_window: false }
    }

    pub fn window_toggle(&mut self, ui: &mut egui::Ui) {
        if ui
            .add(egui::Button::new("Profiler Window").selected(self.show_window))
            .clicked()
        {
            self.show_window = !self.show_window;
        }

        egui::Window::new("Profiler Window")
            .open(&mut self.show_window)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                events_ui(ui);
            });
    }
}
