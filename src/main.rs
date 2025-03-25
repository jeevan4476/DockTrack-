use eframe::egui;
use rdev::{listen, Event, EventType};
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use chrono::Local;
use log::{error, info, warn};
use simple_logger::SimpleLogger;

fn main() -> Result<(), eframe::Error> {
    // Initialize the logger
    SimpleLogger::new().init().unwrap();
    info!("Logger initialized.");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 200.0]), // Set initial window size
        ..Default::default()
    };

    info!("Starting application...");
    eframe::run_native(
        "Task Recorder",
        options,
        Box::new(|_cc| {
            info!("Application window created.");
            Ok(Box::new(TaskRecorderApp::default()))
        }),
    )
}

struct TaskRecorderApp {
    task_name: String,
    is_recording: bool,
    events: Arc<Mutex<Vec<EventData>>>,
}

struct EventData {
    event_type: String,
    button_or_key: String,
    action: String,
    position: String,
    timestamp: String,
}

impl Default for TaskRecorderApp {
    fn default() -> Self {
        info!("TaskRecorderApp initialized.");
        Self {
            task_name: String::new(),
            is_recording: false,
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl TaskRecorderApp {
    fn start_recording(&mut self) {
        info!("Starting recording...");
        self.is_recording = true;
        self.events.lock().unwrap().clear();

        let events = Arc::clone(&self.events);
        thread::spawn(move || {
            info!("Event listener thread started.");
            if let Err(error) = listen(move |event| {
                if let Some(event_data) = process_event(event) {
                    events.lock().unwrap().push(event_data);
                }
            }) {
                error!("Error in event listener: {:?}", error);
            }
            warn!("Event listener thread exited.");
        });
    }

    fn stop_recording(&mut self) {
        info!("Stopping recording...");
        self.is_recording = false;
        self.save_to_csv();
    }

    fn save_to_csv(&self) {
        let filename = format!("{}_events.csv", self.task_name);
        info!("Saving events to file: {}", filename);

        let mut file = match File::create(&filename) {
            Ok(file) => file,
            Err(err) => {
                error!("Failed to create CSV file: {:?}", err);
                return;
            }
        };

        let events = self.events.lock().unwrap();
        if let Err(err) = writeln!(file, "Event Type,Button/Key,Action,Position,Timestamp") {
            error!("Failed to write to CSV file: {:?}", err);
            return;
        }

        for event in events.iter() {
            if let Err(err) = writeln!(
                file,
                "{},({}),{},{},{}",
                event.event_type, event.button_or_key, event.action, event.position, event.timestamp
            ) {
                error!("Failed to write event to CSV file: {:?}", err);
                return;
            }
        }

        info!("Events saved to {}", filename);
    }
}

fn process_event(event: Event) -> Option<EventData> {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    match event.event_type {
        EventType::KeyPress(key) => {
            info!("Key pressed: {:?}", key);
            Some(EventData {
                event_type: "Keyboard".to_string(),
                button_or_key: format!("{:?}", key),
                action: "Pressed".to_string(),
                position: "N/A".to_string(),
                timestamp,
            })
        }
        EventType::KeyRelease(key) => {
            info!("Key released: {:?}", key);
            Some(EventData {
                event_type: "Keyboard".to_string(),
                button_or_key: format!("{:?}", key),
                action: "Released".to_string(),
                position: "N/A".to_string(),
                timestamp,
            })
        }
        EventType::ButtonPress(button) => {
            info!("Mouse button pressed: {:?}", button);
            Some(EventData {
                event_type: "Mouse".to_string(),
                button_or_key: format!("{:?}", button),
                action: "Pressed".to_string(),
                position: "N/A".to_string(), // Mouse position is not available here
                timestamp,
            })
        }
        EventType::ButtonRelease(button) => {
            info!("Mouse button released: {:?}", button);
            Some(EventData {
                event_type: "Mouse".to_string(),
                button_or_key: format!("{:?}", button),
                action: "Released".to_string(),
                position: "N/A".to_string(), // Mouse position is not available here
                timestamp,
            })
        }
        EventType::MouseMove { x, y } => {
            info!("Mouse moved to: ({}, {})", x, y);
            Some(EventData {
                event_type: "Mouse".to_string(),
                button_or_key: "Move".to_string(),
                action: "Moved".to_string(),
                position: format!("({}, {})", x, y),
                timestamp,
            })
        }
        _ => None,
    }
}

impl eframe::App for TaskRecorderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Task Recorder");

            ui.horizontal(|ui| {
                ui.label("Task Name:");
                ui.text_edit_singleline(&mut self.task_name);
            });

            if ui.button("Create Task").clicked() {
                if self.task_name.is_empty() {
                    warn!("Task name is empty.");
                    ui.label("Please enter a task name.");
                } else {
                    self.start_recording();
                }
            }

            if ui.button("Stop Task").clicked() {
                self.stop_recording();
            }

            if self.is_recording {
                ui.label("Recording...");
            }
        });
    }
}