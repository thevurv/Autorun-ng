use std::{
	sync::{Arc, RwLock},
	time::Duration,
};

mod commands;
use commands::{CommandContext, CommandRegistry};

use eframe::{
	CreationContext,
	egui::{
		self, Button, ComboBox, FontId, IconData, TextEdit, TextFormat, Ui, ViewportBuilder,
		text::LayoutJob,
	},
	epaint::Color32,
};
use egui_extras::syntax_highlighting::CodeTheme;

use crate::backend::{Autorun, AutorunStatus};
use autorun_types::Realm;

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
	realm_state: Realm,
	last_update: std::time::Instant,

	// Command system
	command_registry: CommandRegistry,

	// UI state
	input_id: egui::Id,
}

impl Default for App {
	fn default() -> Self {
		Self {
			autorun: Autorun::default(),
			input: String::default(),
			code: String::default(),
			log: Arc::new(RwLock::new(String::new())),
			console_mode: ConsoleMode::default(),
			realm_state: Realm::Menu,
			last_update: std::time::Instant::now(),
			command_registry: CommandRegistry::new(),
			input_id: egui::Id::new("terminal_input"),
		}
	}
}

impl App {
	pub fn new(cc: &CreationContext, autorun: Autorun) -> Self {
		cc.egui_ctx.request_repaint_after(REPAINT_TIME);

		// Set dark theme
		let mut style = (*cc.egui_ctx.style()).clone();
		style.visuals.dark_mode = true;
		style.visuals.window_fill = Color32::from_rgb(25, 25, 25);
		style.visuals.panel_fill = Color32::from_rgb(30, 30, 30);
		style.visuals.extreme_bg_color = Color32::from_rgb(15, 15, 15);
		style.visuals.faint_bg_color = Color32::from_rgb(40, 40, 40);
		style.visuals.code_bg_color = Color32::from_rgb(20, 20, 20);
		style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(35, 35, 35);
		style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(45, 45, 45);
		style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(55, 55, 55);
		style.visuals.widgets.active.bg_fill = Color32::from_rgb(65, 65, 65);
		style.visuals.selection.bg_fill = Color32::from_rgb(70, 130, 180);
		style.visuals.override_text_color = Some(Color32::from_rgb(230, 230, 230));
		style.visuals.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(200, 200, 200);
		style.visuals.widgets.inactive.fg_stroke.color = Color32::from_rgb(180, 180, 180);
		style.visuals.widgets.hovered.fg_stroke.color = Color32::WHITE;
		style.visuals.widgets.active.fg_stroke.color = Color32::WHITE;
		style.visuals.window_stroke = egui::Stroke::new(1.0, Color32::from_rgb(60, 60, 60));
		style.visuals.widgets.noninteractive.bg_stroke =
			egui::Stroke::new(1.0, Color32::from_rgb(50, 50, 50));
		style.visuals.widgets.inactive.bg_stroke =
			egui::Stroke::new(1.0, Color32::from_rgb(70, 70, 70));
		style.visuals.widgets.hovered.bg_stroke =
			egui::Stroke::new(1.0, Color32::from_rgb(100, 100, 100));
		style.visuals.widgets.active.bg_stroke =
			egui::Stroke::new(1.0, Color32::from_rgb(120, 120, 120));
		style.visuals.window_rounding = egui::Rounding::same(8.0);
		style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(6.0);
		style.visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
		style.visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
		style.visuals.widgets.active.rounding = egui::Rounding::same(6.0);
		cc.egui_ctx.set_style(style);

		let log = Arc::new(RwLock::new(String::new()));
		let log_thread = Arc::clone(&log);

		let mut stdio = shh::stdout().unwrap();
		let mut stderr = shh::stderr().unwrap();

		const WAIT_TIME: Duration = Duration::from_millis(200);

		// Background thread to read stdout/stderr to console
		let ctx = cc.egui_ctx.clone();
		std::thread::spawn(move || {
			loop {
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
			}
		});

		let command_registry = CommandRegistry::new();
		// Uncomment the line below to enable additional example commands like console, echo and time
		// command_registry.register_custom_commands();

		Self {
			log,
			autorun,
			command_registry,
			last_update: std::time::Instant::now(),
			input_id: egui::Id::new("terminal_input"),
			..Default::default()
		}
	}

	fn show(&mut self, ui: &mut Ui) {
		ui.horizontal(|ui| {
			ui.heading("Autorun-next");

			match self.autorun.status() {
				AutorunStatus::Disconnected => {
					ui.colored_label(Color32::from_rgb(255, 100, 100), "Disconnected");

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

					#[cfg(debug_assertions)]
					if ui.button("Test ANSI Colors").clicked() {
						self.demo_ansi_colors();
					}
				}
				AutorunStatus::Connected => {
					ui.colored_label(Color32::from_rgb(100, 255, 100), "Connected");

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
				match self.console_mode {
					ConsoleMode::Terminal => {
						let log_content = self.log.read().unwrap().clone();
						self.render_ansi_text(ui, &log_content);
					}
					ConsoleMode::Executor => {
						// Allocate fixed space for the code editor (same as terminal)
						let editor_size = egui::Vec2::new(HALF.0, SIZE.1 * 0.7 - 4.0);
						let (rect, _response) =
							ui.allocate_exact_size(editor_size, egui::Sense::hover());

						ui.allocate_ui_at_rect(rect, |ui| {
							let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
								let mut layout_job = egui_extras::syntax_highlighting::highlight(
									ui.ctx(),
									&CodeTheme::dark(),
									string,
									"lua",
								);
								layout_job.wrap.max_width = wrap_width;
								ui.fonts(|f| f.layout_job(layout_job))
							};

							// Style the text editor background and hint text
							ui.style_mut().visuals.extreme_bg_color = Color32::BLACK;
							ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::BLACK;
							ui.style_mut().visuals.widgets.noninteractive.bg_fill = Color32::BLACK;
							ui.style_mut().visuals.widgets.hovered.bg_fill = Color32::BLACK;
							ui.style_mut().visuals.widgets.active.bg_fill = Color32::BLACK;
							ui.style_mut().visuals.widgets.inactive.fg_stroke.color =
								Color32::from_rgb(120, 120, 120);
							ui.style_mut().visuals.widgets.inactive.bg_stroke =
								egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 80));
							ui.style_mut().visuals.widgets.noninteractive.bg_stroke =
								egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 80));
							ui.style_mut().visuals.widgets.hovered.bg_stroke =
								egui::Stroke::new(1.0, Color32::from_rgb(100, 100, 100));
							ui.style_mut().visuals.widgets.active.bg_stroke =
								egui::Stroke::new(1.0, Color32::from_rgb(120, 120, 120));
							ui.style_mut().visuals.widgets.inactive.rounding =
								egui::Rounding::same(8.0);
							ui.style_mut().visuals.widgets.noninteractive.rounding =
								egui::Rounding::same(8.0);
							ui.style_mut().visuals.widgets.hovered.rounding =
								egui::Rounding::same(8.0);
							ui.style_mut().visuals.widgets.active.rounding =
								egui::Rounding::same(8.0);

							ui.add_sized(
								editor_size,
								egui::TextEdit::multiline(&mut self.code)
									.font(egui::TextStyle::Monospace)
									.code_editor()
									.desired_width(f32::INFINITY)
									.layouter(&mut layouter),
							);
						});
					}
				};

				ui.horizontal(|ui| {
					// Mode dropdown takes 20% of width
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

					// Show realm dropdown only in executor mode
					if self.console_mode == ConsoleMode::Executor {
						ComboBox::from_id_source("RealmState")
							.width(HALF.0 * 0.15)
							.selected_text(self.realm_state.to_string())
							.show_ui(ui, |ui| {
								for realm in [Realm::Menu, Realm::Client] {
									if realm != self.realm_state {
										ui.selectable_value(
											&mut self.realm_state,
											realm,
											realm.to_string(),
										);
									}
								}
							});
					}

					let input_width = if self.console_mode == ConsoleMode::Executor {
						HALF.0 * 0.4825 - 10.0 // Shorter width in executor mode to align with terminal mode
					} else {
						HALF.0 * 0.65 - 10.0 // Full width in terminal mode
					};

					let input_response = TextEdit::singleline(&mut self.input)
						.desired_width(input_width)
						.interactive(self.console_mode == ConsoleMode::Terminal)
						.id(self.input_id)
						.show(ui);

					// Check if Enter was pressed to submit command
					let should_submit = input_response.response.lost_focus()
						&& input_response
							.response
							.ctx
							.input(|i| i.key_pressed(egui::Key::Enter))
						&& !self.input.is_empty()
						&& self.console_mode == ConsoleMode::Terminal;

					if should_submit {
						// Execute command
						let mut context = CommandContext::new(Arc::clone(&self.log), &self.autorun);

						match self
							.command_registry
							.execute_command(&self.input, &mut context)
						{
							Ok(true) => {
								// Command was handled
							}
							Ok(false) => {
								// Not a recognized command
								context.write_error(&format!("Unknown command: '{}'", self.input));
							}
							Err(e) => {
								eprintln!("Command execution error: {}", e);
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
									// Execute command
									let mut context =
										CommandContext::new(Arc::clone(&self.log), &self.autorun);

									match self
										.command_registry
										.execute_command(&self.input, &mut context)
									{
										Ok(true) => {
											// Command was handled
										}
										Ok(false) => {
											// Not a recognized command
											context.write_error(&format!(
												"Unknown command: '{}'",
												self.input
											));
										}
										Err(e) => {
											eprintln!("Command execution error: {}", e);
										}
									}
									self.input = String::new();
								}
							}
							ConsoleMode::Executor => {
								if !self.code.is_empty() {
									if let Err(e) =
										self.autorun.run_code(self.realm_state, &self.code)
									{
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

	fn render_ansi_text(&self, ui: &mut Ui, text: &str) {
		let segments = parse_ansi_text(text);

		// Create a frame with terminal-like styling with fixed size
		let frame = egui::Frame::default()
			.fill(Color32::BLACK)
			.stroke(egui::Stroke::new(1.0, Color32::from_rgb(80, 80, 80)))
			.rounding(egui::Rounding::same(8.0))
			.inner_margin(egui::Margin::same(8.0))
			.shadow(egui::Shadow {
				offset: egui::Vec2::new(2.0, 2.0),
				blur: 4.0,
				spread: 0.0,
				color: Color32::from_black_alpha(80),
			});

		// Allocate fixed space for the terminal
		let terminal_size = egui::Vec2::new(HALF.0, SIZE.1 * 0.7);
		let (rect, _response) = ui.allocate_exact_size(terminal_size, egui::Sense::hover());

		// Show the frame in the allocated space
		ui.allocate_ui_at_rect(rect, |ui| {
			frame.show(ui, |ui| {
				egui::ScrollArea::vertical()
					.min_scrolled_height(SIZE.1 * 0.7 - 20.0) // Account for frame margins
					.max_height(SIZE.1 * 0.7 - 20.0)
					.auto_shrink([false, false])
					.stick_to_bottom(true)
					.show(ui, |ui| {
						ui.set_width(HALF.0 - 20.0); // Account for frame margins
						ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

						// Group segments by lines to render them inline
						let mut current_job = LayoutJob::default();
						let font_id = FontId::monospace(12.0);

						for segment in segments {
							if segment.text.is_empty() {
								continue;
							}

							// Split by newlines and handle each line
							let lines: Vec<&str> = segment.text.split('\n').collect();
							for (line_idx, line) in lines.iter().enumerate() {
								if line_idx > 0 {
									// Finish current line and start a new one
									if !current_job.text.is_empty() {
										ui.label(current_job.clone());
										current_job = LayoutJob::default();
									}
								}

								if !line.is_empty() {
									// Add this line segment to the current job
									let color = segment.color.unwrap_or(Color32::WHITE);
									let mut text_format = TextFormat {
										font_id: font_id.clone(),
										color,
										..Default::default()
									};

									// Set background color if available
									if let Some(bg_color) = segment.background {
										text_format.background = bg_color;
									}

									current_job.append(line, 0.0, text_format);
								}
							}
						}

						// Render any remaining text
						if !current_job.text.is_empty() {
							ui.label(current_job);
						}
					});
			});
		});
	}

	#[cfg(debug_assertions)]
	fn demo_ansi_colors(&self) {
		let demo_text = format!(
			"{}ANSI Color Demo:{}\n\
			{}Red text{} - {}Green text{} - {}Blue text{}\n\
			{}Yellow{} - {}Magenta{} - {}Cyan{}\n\
			{}Bright Red{} - {}Bright Green{} - {}Bright Blue{}\n\
			{}Red on Yellow{} - {}White on Blue{} - {}Black on Cyan{}\n\
			{}Normal text again{}",
			"\x1b[1m",
			"\x1b[0m", // Bold title and reset
			"\x1b[31m",
			"\x1b[0m",
			"\x1b[32m",
			"\x1b[0m",
			"\x1b[34m",
			"\x1b[0m", // Red, Green, Blue
			"\x1b[33m",
			"\x1b[0m",
			"\x1b[35m",
			"\x1b[0m",
			"\x1b[36m",
			"\x1b[0m", // Yellow, Magenta, Cyan
			"\x1b[91m",
			"\x1b[0m",
			"\x1b[92m",
			"\x1b[0m",
			"\x1b[94m",
			"\x1b[0m", // Bright Red, Green, Blue
			"\x1b[31;43m",
			"\x1b[0m",
			"\x1b[37;44m",
			"\x1b[0m",
			"\x1b[30;46m",
			"\x1b[0m", // Foreground + background combinations
			"\x1b[37m",
			"\x1b[0m" // White and reset
		);

		// Add the demo text to the log
		if let Ok(mut log) = self.log.write() {
			log.push_str(&demo_text);
			log.push('\n');
		}
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

#[derive(Debug, Clone)]
struct TextSegment {
	text: String,
	color: Option<Color32>,
	background: Option<Color32>,
}

fn parse_ansi_text(text: &str) -> Vec<TextSegment> {
	let mut segments = Vec::new();
	let mut current_text = String::new();
	let mut current_color = None;
	let mut current_background = None;
	let mut chars = text.chars().peekable();

	while let Some(ch) = chars.next() {
		if ch == '\x1b' && chars.peek() == Some(&'[') {
			// Found ANSI escape sequence
			if !current_text.is_empty() {
				segments.push(TextSegment {
					text: current_text.clone(),
					color: current_color,
					background: current_background,
				});
				current_text.clear();
			}

			chars.next(); // consume '['
			let mut code = String::new();

			// Read until 'm'
			while let Some(ch) = chars.next() {
				if ch == 'm' {
					break;
				}
				code.push(ch);
			}

			// Parse the color code - handle multiple codes separated by semicolons
			let codes: Vec<&str> = code.split(';').collect();
			for single_code in codes {
				if single_code.trim() == "0" {
					current_color = None; // Reset
					current_background = None;
				} else if let Some(color) = parse_ansi_foreground_color(single_code) {
					current_color = Some(color);
				} else if let Some(bg_color) = parse_ansi_background_color(single_code) {
					current_background = Some(bg_color);
				}
			}
		} else {
			current_text.push(ch);
		}
	}

	// Add remaining text
	if !current_text.is_empty() {
		segments.push(TextSegment {
			text: current_text,
			color: current_color,
			background: current_background,
		});
	}

	segments
}

fn parse_ansi_foreground_color(code: &str) -> Option<Color32> {
	match code.trim() {
		"0" => None, // Reset
		// Foreground colors
		"30" => Some(Color32::BLACK),
		"31" => Some(Color32::RED),
		"32" => Some(Color32::GREEN),
		"33" => Some(Color32::YELLOW),
		"34" => Some(Color32::BLUE),
		"35" => Some(Color32::from_rgb(255, 0, 255)), // Magenta
		"36" => Some(Color32::from_rgb(0, 255, 255)), // Cyan
		"37" => Some(Color32::WHITE),
		// Bright foreground colors
		"90" => Some(Color32::DARK_GRAY),
		"91" => Some(Color32::from_rgb(255, 100, 100)), // Bright red
		"92" => Some(Color32::from_rgb(100, 255, 100)), // Bright green
		"93" => Some(Color32::from_rgb(255, 255, 100)), // Bright yellow
		"94" => Some(Color32::from_rgb(100, 100, 255)), // Bright blue
		"95" => Some(Color32::from_rgb(255, 100, 255)), // Bright magenta
		"96" => Some(Color32::from_rgb(100, 255, 255)), // Bright cyan
		"97" => Some(Color32::from_rgb(240, 240, 240)), // Bright white
		_ => None,
	}
}

fn parse_ansi_background_color(code: &str) -> Option<Color32> {
	match code.trim() {
		// Background colors
		"40" => Some(Color32::BLACK),
		"41" => Some(Color32::RED),
		"42" => Some(Color32::GREEN),
		"43" => Some(Color32::YELLOW),
		"44" => Some(Color32::BLUE),
		"45" => Some(Color32::from_rgb(255, 0, 255)), // Magenta
		"46" => Some(Color32::from_rgb(0, 255, 255)), // Cyan
		"47" => Some(Color32::WHITE),
		// Bright background colors
		"100" => Some(Color32::DARK_GRAY),
		"101" => Some(Color32::from_rgb(255, 100, 100)), // Bright red
		"102" => Some(Color32::from_rgb(100, 255, 100)), // Bright green
		"103" => Some(Color32::from_rgb(255, 255, 100)), // Bright yellow
		"104" => Some(Color32::from_rgb(100, 100, 255)), // Bright blue
		"105" => Some(Color32::from_rgb(255, 100, 255)), // Bright magenta
		"106" => Some(Color32::from_rgb(100, 255, 255)), // Bright cyan
		"107" => Some(Color32::from_rgb(240, 240, 240)), // Bright white
		_ => None,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_ansi_color_parsing() {
		let test_text = "\x1b[31mRed text\x1b[0m normal \x1b[32mGreen text\x1b[0m";
		let segments = parse_ansi_text(test_text);

		assert_eq!(segments.len(), 3);
		assert_eq!(segments[0].text, "Red text");
		assert_eq!(segments[0].color, Some(Color32::RED));
		assert_eq!(segments[0].background, None);
		assert_eq!(segments[1].text, " normal ");
		assert_eq!(segments[1].color, None);
		assert_eq!(segments[2].text, "Green text");
		assert_eq!(segments[2].color, Some(Color32::GREEN));
	}

	#[test]
	fn test_ansi_newlines() {
		let test_text = "\x1b[31mLine 1\nLine 2\x1b[0m\nLine 3";
		let segments = parse_ansi_text(test_text);

		// Should have 2 segments: colored text with newlines, and normal text
		assert_eq!(segments.len(), 2);
		assert!(segments[0].text.contains("Line 1\nLine 2"));
		assert_eq!(segments[0].color, Some(Color32::RED));
		assert!(segments[1].text.contains("Line 3"));
		assert_eq!(segments[1].color, None);
	}

	#[test]
	fn test_ansi_multiple_colors() {
		let test_text = "\x1b[31;42mRed on green\x1b[0m";
		let segments = parse_ansi_text(test_text);

		assert_eq!(segments.len(), 1);
		assert_eq!(segments[0].text, "Red on green");
		// Should have red foreground and green background
		assert_eq!(segments[0].color, Some(Color32::RED));
		assert_eq!(segments[0].background, Some(Color32::GREEN));
	}
}
