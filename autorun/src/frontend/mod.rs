pub mod error;

use std::{
	sync::{Arc, RwLock},
	time::Duration,
};

use eframe::{
	egui::{self, Button, ComboBox, TextEdit, Ui},
	epaint::Color32,
	CreationContext,
};

use crate::backend::Autorun;

const SIZE: (f32, f32) = (900.0, 500.0);
const HALF: (f32, f32) = (SIZE.0 / 2.0, SIZE.1 / 2.0);
const REPAINT_TIME: Duration = Duration::from_secs(2);

pub fn run(autorun: Autorun) {
	eframe::run_native(
		"Autorun",
		eframe::NativeOptions {
			max_window_size: Some(SIZE.into()),
			min_window_size: Some(HALF.into()),

			..Default::default()
		},
		Box::new(|cc| Box::new(App::new(cc, autorun))),
	);
}

#[derive(Copy, Clone, PartialEq)]
enum ConsoleMode {
	Terminal,
	Executor,
}

impl ConsoleMode {
	fn as_str(&self) -> &str {
		match *self {
			ConsoleMode::Terminal => "Terminal",
			ConsoleMode::Executor => "Executor",
		}
	}
}

impl std::fmt::Display for ConsoleMode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ConsoleMode::Terminal => write!(f, "Terminal"),
			ConsoleMode::Executor => write!(f, "Executor"),
		}
	}
}

impl Default for ConsoleMode {
	fn default() -> Self {
		ConsoleMode::Terminal
	}
}

#[derive(Default)]
struct App {
	// State
	autorun: Autorun,

	// Command input
	input: String,
	code: String,

	log: Arc<RwLock<String>>,

	console_mode: ConsoleMode,
}

impl App {
	fn new(cc: &CreationContext, autorun: Autorun) -> Self {
		cc.egui_ctx.request_repaint_after(REPAINT_TIME);

		let log = Arc::new(RwLock::new(String::new()));
		let log_thread = Arc::clone(&log);

		let mut stdio = shh::stdout().unwrap();
		let mut stderr = shh::stderr().unwrap();

		const WAIT_TIME: Duration = Duration::from_secs(1);

		#[allow(unused_must_use)]
		std::thread::spawn(move || loop {
			use std::io::Read;

			std::thread::sleep(WAIT_TIME);

			let mut log = log_thread.write().unwrap();
			stdio.read_to_string(&mut log);
			stderr.read_to_string(&mut log);
		});

		Self {
			log,
			autorun,
			..Default::default()
		}
	}

	fn show(&mut self, ui: &mut Ui) {
		ui.horizontal(|ui| {
			ui.heading("Autorun-rs");

			const RED: Color32 = Color32::from_rgb(255, 0, 0);
			const GREEN: Color32 = Color32::from_rgb(0, 255, 0);

			if self.autorun.status().is_ready() {
				ui.colored_label(GREEN, "Loaded");
			} else {
				ui.colored_label(RED, "Unloaded");
			}
		});

		ui.separator();

		ui.horizontal(|ui| {
			// Left side
			ui.vertical(|ui| {
				egui::ScrollArea::vertical()
					.min_scrolled_height(SIZE.1 * 0.7)
					.auto_shrink([false, true])
					.show(ui, |ui| {
						match self.console_mode {
							ConsoleMode::Terminal => {
								let mut x = self.log.read().unwrap().clone();
								TextEdit::multiline(&mut x)
									.code_editor()
									.interactive(false)
									.desired_rows(22)
									.desired_width(HALF.0)
									.show(ui);
							}
							ConsoleMode::Executor => {
								TextEdit::multiline(&mut self.code)
									.code_editor()
									.hint_text("print('Hello Autorun')")
									.desired_rows(22)
									.desired_width(HALF.0)
									.show(ui);
							}
						};
					});

				ui.horizontal(|ui| {
					// 20% of width goes to dropdown.
					// 10% goes to the button
					// 70% goes to input box.

					ComboBox::from_id_source("ConsoleMode")
						.width(HALF.0 * 0.2)
						.selected_text(self.console_mode.as_str())
						.show_ui(ui, |ui| {
							for mode in [ConsoleMode::Terminal, ConsoleMode::Executor] {
								if mode != self.console_mode {
									ui.selectable_value(
										&mut self.console_mode,
										mode,
										mode.as_str(),
									);
								}
							}
						});

					let input_box = TextEdit::singleline(&mut self.input)
						.desired_width(HALF.0 * 0.65 - 10.0) // Magic numbers. woop
						.show(ui);

					if input_box.response.lost_focus() {
						println!("{}", self.input);
						self.input = String::new();
					}

					if ui
						.add_sized([0.0, ui.available_height()], Button::new("Execute"))
						.clicked()
					{
						println!("Todo: Run code {}", self.code);
					}
				});
			});
		});
	}
}

impl eframe::App for App {
	fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
		self.autorun.detach();
	}

	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show(ctx, |ui| self.show(ui));
	}
}
