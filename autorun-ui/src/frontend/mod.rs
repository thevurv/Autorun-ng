use std::{
	sync::{Arc, RwLock},
	time::Duration,
};

use eframe::{
	egui::{self, Button, ComboBox, IconData, TextEdit, Ui, ViewportBuilder},
	epaint::Color32,
	CreationContext,
};

use crate::backend::{Autorun, AutorunStatus};

const SIZE: (f32, f32) = (900.0, 500.0);
const HALF: (f32, f32) = (SIZE.0 / 2.0, SIZE.1 / 2.0);
const REPAINT_TIME: Duration = Duration::from_secs(2);
const UPDATE_INTERVAL: Duration = Duration::from_millis(500);

fn load_icon() -> Option<std::sync::Arc<IconData>> {
	let icon_bytes = include_bytes!("../../../assets/run.png");

	match image::load_from_memory(icon_bytes) {
		Ok(img) => {
			let img = img.to_rgba8();
			let (width, height) = img.dimensions();
			Some(std::sync::Arc::new(IconData {
				rgba: img.into_raw(),
				width,
				height,
			}))
		}
		Err(_) => None,
	}
}

pub fn run(autorun: Autorun) {
	let icon = load_icon();

	let _ = eframe::run_native(
		"Autorun",
		eframe::NativeOptions {
			viewport: if let Some(icon) = icon {
				ViewportBuilder::default().with_icon(icon)
			} else {
				ViewportBuilder::default()
			},

			..Default::default()
		},
		Box::new(|cc| Ok(Box::new(App::new(cc, autorun)))),
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

struct App {
	// State
	autorun: Autorun,

	// Command input
	input: String,
	code: String,

	log: Arc<RwLock<String>>,

	console_mode: ConsoleMode,
	last_update: std::time::Instant,
}

impl Default for App {
	fn default() -> Self {
		Self {
			autorun: Autorun::default(),
			input: String::default(),
			code: String::default(),
			log: Arc::new(RwLock::new(String::new())),
			console_mode: ConsoleMode::default(),
			last_update: std::time::Instant::now(),
		}
	}
}

impl App {
	pub fn new(cc: &CreationContext, autorun: Autorun) -> Self {
		cc.egui_ctx.request_repaint_after(REPAINT_TIME);

		let log = Arc::new(RwLock::new(String::new()));
		let log_thread = Arc::clone(&log);

		let mut stdio = shh::stdout().unwrap();
		let mut stderr = shh::stderr().unwrap();

		const WAIT_TIME: Duration = Duration::from_millis(200);

		// Background thread to read stdout/stderr to console
		let ctx = cc.egui_ctx.clone();
		std::thread::spawn(move || loop {
			use std::io::Read;

			std::thread::sleep(WAIT_TIME);

			let mut log = log_thread.write().unwrap();
			match (
				stdio.read_to_string(&mut log),
				stderr.read_to_string(&mut log),
			) {
				(Ok(_), Ok(_)) | (Ok(_), _) | (_, Ok(_)) => ctx.request_repaint(),
				_ => (),
			}

			ctx.request_repaint();
		});

		Self {
			log,
			autorun,
			last_update: std::time::Instant::now(),
			..Default::default()
		}
	}

	fn show(&mut self, ui: &mut Ui) {
		ui.horizontal(|ui| {
			ui.heading("Autorun-next");

			match self.autorun.status() {
				AutorunStatus::Disconnected => {
					ui.colored_label(Color32::RED, "Disconnected");

					if ui.button("Launch").clicked() {
						if let Err(e) = self.autorun.start_attached() {
							eprintln!("Failed to start attached: {}", e);
						}
					}

					if ui.button("Connect").clicked() {
						if let Err(e) = self.autorun.try_connect_to_game() {
							eprintln!("Failed to connect: {}", e);
						}
					}
				}
				AutorunStatus::Connected => {
					ui.colored_label(Color32::GREEN, "Connected");

					if ui.button("Disconnect").clicked() {
						if let Err(e) = self.autorun.detach() {
							eprintln!("Failed to detach: {}", e);
						}
					}
				}
			};
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

					if input_box.response.lost_focus() && !self.input.is_empty() {
						match self.console_mode {
							ConsoleMode::Terminal => {
								// Send command to game console
								if let Err(e) = self.autorun.print_to_game(&self.input) {
									eprintln!("Failed to send command: {}", e);
								}
							}
							ConsoleMode::Executor => {
								// Execute as Lua code
								if let Err(e) = self.autorun.run_code(&self.input) {
									eprintln!("Failed to execute code: {}", e);
								}
							}
						}
						self.input = String::new();
					}

					if ui
						.add_sized([0.0, ui.available_height()], Button::new("Execute"))
						.clicked()
					{
						match self.console_mode {
							ConsoleMode::Terminal => {
								if !self.input.is_empty() {
									if let Err(e) = self.autorun.print_to_game(&self.input) {
										eprintln!("Failed to send command: {}", e);
									}
									self.input = String::new();
								}
							}
							ConsoleMode::Executor => {
								if !self.code.is_empty() {
									if let Err(e) = self.autorun.run_code(&self.code) {
										eprintln!("Failed to execute code: {}", e);
									}
								}
							}
						}
					}
				});
			});
		});
	}
}

impl eframe::App for App {
	fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
		let _ = self.autorun.detach();
	}

	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		// Only update autorun status periodically to avoid blocking UI
		if self.last_update.elapsed() >= UPDATE_INTERVAL {
			self.autorun.update();
			self.last_update = std::time::Instant::now();
		}

		egui::CentralPanel::default().show(ctx, |ui| self.show(ui));
	}
}
