use std::{
	sync::{Arc, RwLock},
	time::Duration,
};

mod commands;
use autorun_log::*;
use commands::{CommandContext, CommandRegistry};

use eframe::{
	CreationContext,
	egui::{
		self, Button, Color32, ComboBox, FontId, Frame, IconData, Margin, Rounding, Shadow, Stroke, TextEdit, TextFormat, Ui,
		Vec2, ViewportBuilder, text::LayoutJob,
	},
	epaint::FontFamily,
};
use egui_extras::syntax_highlighting::CodeTheme;

use crate::backend::{Autorun, AutorunStatus};
use autorun_types::Realm;

const SIZE: (f32, f32) = (1200.0, 700.0);
const REPAINT_TIME: Duration = Duration::from_secs(2);
const UPDATE_INTERVAL: Duration = Duration::from_millis(500);

fn load_icon() -> Option<std::sync::Arc<IconData>> {
	let icon_bytes = include_bytes!("../../../../assets/run.png");

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
		"Autorun-ng",
		eframe::NativeOptions {
			viewport: if let Some(icon) = icon {
				ViewportBuilder::default()
					.with_icon(icon)
					.with_inner_size(SIZE)
					.with_min_inner_size([800.0, 500.0])
			} else {
				ViewportBuilder::default()
					.with_inner_size(SIZE)
					.with_min_inner_size([800.0, 500.0])
			},
			..Default::default()
		},
		Box::new(|cc| Ok(Box::new(App::new(cc, autorun)))),
	);
}

#[derive(Copy, Clone, Default, PartialEq)]
enum ActiveTab {
	#[default]
	Console,
	Settings,
	About,
}

impl ActiveTab {
	fn as_str(&self) -> &str {
		match *self {
			ActiveTab::Console => "Console",
			ActiveTab::Settings => "Settings",
			ActiveTab::About => "About",
		}
	}
}

struct App {
	// State
	autorun: Autorun,

	// UI State
	active_tab: ActiveTab,
	panel_split_ratio: f32,

	// Terminal
	terminal_input: String,
	log: Arc<RwLock<String>>,

	// Executor
	code: String,
	realm_state: Realm,

	// System
	last_update: std::time::Instant,
	command_registry: CommandRegistry,
	terminal_input_id: egui::Id,
	user_disconnected: bool,
}

impl App {
	fn validate_plugins(autorun: &Autorun) -> anyhow::Result<()> {
		let (plugins, errors) = autorun.workspace().get_plugins()?;

		info!("Loaded {} plugins successfully.", plugins.len());

		for error in &errors {
			error!("Failed to load plugin: {error}");
		}

		Ok(())
	}

	pub fn new(cc: &CreationContext, autorun: Autorun) -> Self {
		cc.egui_ctx.request_repaint_after(REPAINT_TIME);

		// Set modern dark theme
		let mut style = (*cc.egui_ctx.style()).clone();
		style.visuals.dark_mode = true;
		style.visuals.window_fill = Color32::from_rgb(20, 20, 20);
		style.visuals.panel_fill = Color32::from_rgb(25, 25, 25);
		style.visuals.extreme_bg_color = Color32::from_rgb(15, 15, 15);
		style.visuals.faint_bg_color = Color32::from_rgb(35, 35, 35);
		style.visuals.code_bg_color = Color32::from_rgb(18, 18, 18);
		style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(30, 30, 30);
		style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(40, 40, 40);
		style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(50, 50, 50);
		style.visuals.widgets.active.bg_fill = Color32::from_rgb(60, 60, 60);
		style.visuals.selection.bg_fill = Color32::from_rgb(70, 130, 180);
		style.visuals.override_text_color = Some(Color32::from_rgb(235, 235, 235));
		style.visuals.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(200, 200, 200);
		style.visuals.widgets.inactive.fg_stroke.color = Color32::from_rgb(180, 180, 180);
		style.visuals.widgets.hovered.fg_stroke.color = Color32::WHITE;
		style.visuals.widgets.active.fg_stroke.color = Color32::WHITE;
		style.visuals.window_stroke = Stroke::new(1.0, Color32::from_rgb(60, 60, 60));
		style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(45, 45, 45));
		style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(55, 55, 55));
		style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(80, 80, 80));
		style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(100, 100, 100));
		style.visuals.window_rounding = Rounding::same(6.0);
		style.visuals.widgets.noninteractive.rounding = Rounding::same(4.0);
		style.visuals.widgets.inactive.rounding = Rounding::same(4.0);
		style.visuals.widgets.hovered.rounding = Rounding::same(4.0);
		style.visuals.widgets.active.rounding = Rounding::same(4.0);
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
				match (stdio.read_to_string(&mut log), stderr.read_to_string(&mut log)) {
					(Ok(_), Ok(_)) | (Ok(_), _) | (_, Ok(_)) => ctx.request_repaint(),
					_ => (),
				}

				ctx.request_repaint();
			}
		});

		if let Err(why) = Self::validate_plugins(&autorun) {
			error!("Failed to validate plugins: {why}");
		}

		let command_registry = CommandRegistry::new();

		Self {
			log,
			autorun,
			command_registry,
			last_update: std::time::Instant::now(),
			terminal_input_id: egui::Id::new("terminal_input"),
			active_tab: ActiveTab::default(),
			panel_split_ratio: 0.5,
			terminal_input: String::new(),
			code: String::new(),
			realm_state: Realm::Menu,
			user_disconnected: false,
		}
	}

	fn show_header(&mut self, ui: &mut Ui) {
		// Header with title and connection controls
		Frame::default()
			.fill(Color32::from_rgb(30, 30, 30))
			.stroke(Stroke::new(1.0, Color32::from_rgb(50, 50, 50)))
			.inner_margin(Margin::symmetric(12.0, 8.0))
			.show(ui, |ui| {
				ui.horizontal(|ui| {
					ui.heading("Autorun-ng");

					ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
						// Connection controls
						match self.autorun.status() {
							AutorunStatus::Disconnected => {
								if ui.button("Launch").clicked() {
									self.user_disconnected = false;
									if let Err(e) = self.autorun.launch_game() {
										eprintln!("Failed to start attached: {}", e);
									}
								}

								ui.colored_label(Color32::from_rgb(255, 120, 120), "⛔ Disconnected");
							}
							AutorunStatus::Connected => {
								ui.colored_label(Color32::from_rgb(120, 255, 120), "✅ Connected");
							}
						}
					});
				});
			});
	}

	fn show_tab_bar(&mut self, ui: &mut Ui) {
		// Tab bar with proper spacing
		ui.add_space(12.0); // Add top margin
		ui.horizontal(|ui| {
			for tab in [ActiveTab::Console, ActiveTab::Settings, ActiveTab::About] {
				let selected = self.active_tab == tab;

				// Create button with generous padding for better click area
				let button_text = format!("{}  {}  {}", "  ", tab.as_str(), "  ");
				let mut button = Button::new(button_text).min_size(egui::Vec2::new(100.0, 36.0)); // Minimum size for better clicking

				if selected {
					// Active tab styling - elevated appearance
					button = button
						.fill(Color32::from_rgb(45, 45, 45))
						.stroke(Stroke::new(2.0, Color32::from_rgb(70, 130, 180)))
						.rounding(Rounding::same(8.0));
				} else {
					// Inactive tab styling - subtle but clickable
					button = button
						.fill(Color32::from_rgb(30, 30, 30))
						.stroke(Stroke::new(1.0, Color32::from_rgb(60, 60, 60)))
						.rounding(Rounding::same(6.0));
				}

				// Add hover effect
				let response = ui.add(button);
				if response.hovered() && !selected {
					// Draw hover overlay
					ui.painter().rect_filled(
						response.rect,
						Rounding::same(6.0),
						Color32::from_rgba_premultiplied(70, 130, 180, 30),
					);
				}

				if response.clicked() {
					self.active_tab = tab;
				}

				ui.add_space(6.0); // Gap between tabs
			}
		});
		ui.add_space(8.0); // Space after tabs
	}

	fn show_console_tab(&mut self, ui: &mut Ui) {
		// Main content area with split panels
		let available_rect = ui.available_rect_before_wrap();
		let content_height = available_rect.height();

		// Calculate panel sizes
		let panel_width = available_rect.width() * self.panel_split_ratio;
		let remaining_width = available_rect.width() - panel_width - 24.0; // Account for splitter and margins

		ui.horizontal(|ui| {
			// Terminal panel
			ui.vertical(|ui| {
				ui.set_width(panel_width);
				ui.set_height(content_height);
				self.show_terminal_panel(ui);
			});

			// Resizable splitter
			let splitter_response = ui.allocate_response(egui::Vec2::new(12.0, content_height), egui::Sense::click_and_drag());

			if splitter_response.dragged() {
				let delta = splitter_response.drag_delta().x;
				let new_ratio = self.panel_split_ratio + (delta / available_rect.width());
				self.panel_split_ratio = new_ratio.clamp(0.2, 0.8);
			}

			// Draw splitter
			ui.painter().rect_filled(
				splitter_response.rect,
				egui::Rounding::same(2.0),
				if splitter_response.hovered() || splitter_response.dragged() {
					Color32::from_rgb(80, 80, 80)
				} else {
					Color32::from_rgb(50, 50, 50)
				},
			);

			// Change cursor when hovering over splitter
			if splitter_response.hovered() {
				ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::ResizeHorizontal);
			}

			// Executor panel
			ui.vertical(|ui| {
				ui.set_width(remaining_width.max(250.0)); // Ensure minimum width
				ui.set_height(content_height);
				self.show_executor_panel(ui);
			});
		});
	}

	fn show_terminal_panel(&mut self, ui: &mut Ui) {
		Frame::default()
			.fill(Color32::from_rgb(22, 22, 22))
			.stroke(Stroke::new(1.0, Color32::from_rgb(50, 50, 50)))
			.rounding(Rounding::same(6.0))
			.inner_margin(Margin::same(12.0))
			.shadow(Shadow {
				offset: Vec2::new(2.0, 2.0),
				blur: 4.0,
				spread: 0.0,
				color: Color32::from_black_alpha(60),
			})
			.show(ui, |ui| {
				ui.vertical(|ui| {
					// Terminal header
					ui.horizontal(|ui| {
						ui.heading("💻 Terminal");
						ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
							if ui.button("🗑 Clear").clicked()
								&& let Ok(mut log) = self.log.write()
							{
								log.clear();
							}
						});
					});

					ui.separator();

					// Terminal output
					let terminal_height = ui.available_height() - 60.0; // Reserve space for input
					let log_content = self.log.read().unwrap().clone();

					let (rect, _response) =
						ui.allocate_exact_size(Vec2::new(ui.available_width(), terminal_height), egui::Sense::hover());

					ui.allocate_ui_at_rect(rect, |ui| {
						Frame::default()
							.fill(Color32::BLACK)
							.stroke(Stroke::new(1.0, Color32::from_rgb(60, 60, 60)))
							.rounding(Rounding::same(4.0))
							.inner_margin(Margin::same(8.0))
							.show(ui, |ui| {
								egui::ScrollArea::vertical()
									.max_height(terminal_height - 20.0)
									.auto_shrink([false, false])
									.stick_to_bottom(true)
									.show(ui, |ui| {
										ui.set_width(ui.available_width());
										self.render_ansi_text(ui, &log_content);
									});
							});
					});

					// Terminal input
					ui.horizontal(|ui| {
						ui.label("$");

						let input_response = TextEdit::singleline(&mut self.terminal_input)
							.desired_width(ui.available_width() - 80.0)
							.font(egui::TextStyle::Monospace)
							.id(self.terminal_input_id)
							.show(ui);

						// Handle Enter key
						let enter_pressed = input_response.response.ctx.input(|i| i.key_pressed(egui::Key::Enter))
							&& !self.terminal_input.is_empty();

						let should_submit =
							(input_response.response.lost_focus() && enter_pressed) || ui.button("Run").clicked();

						if should_submit {
							self.execute_terminal_command();
							// Keep focus on the input after submitting
							if enter_pressed {
								input_response.response.request_focus();
							}
						}
					});
				});
			});
	}

	fn show_executor_panel(&mut self, ui: &mut Ui) {
		Frame::default()
			.fill(Color32::from_rgb(22, 22, 22))
			.stroke(Stroke::new(1.0, Color32::from_rgb(50, 50, 50)))
			.rounding(Rounding::same(6.0))
			.inner_margin(Margin::same(12.0))
			.shadow(Shadow {
				offset: Vec2::new(2.0, 2.0),
				blur: 4.0,
				spread: 0.0,
				color: Color32::from_black_alpha(60),
			})
			.show(ui, |ui| {
				ui.vertical(|ui| {
					// Executor header
					ui.horizontal(|ui| {
						ui.heading("Lua Executor");

						ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
							ui.add_space(8.0); // Add space from right edge

							if ui.button("Execute").clicked() {
								self.execute_lua_code();
							}

							ui.add_space(4.0); // Add space between controls

							// Color-coded realm selector
							let (realm_color, realm_text) = match self.realm_state {
								Realm::Menu => (Color32::from_rgb(100, 200, 100), "Menu"),   // Green
								Realm::Client => (Color32::from_rgb(255, 165, 0), "Client"), // Orange
							};

							ui.scope(|ui| {
								ui.style_mut().visuals.override_text_color = Some(realm_color);
								ComboBox::from_id_source("realm_selector")
									.width(110.0)
									.selected_text(realm_text)
									.show_ui(ui, |ui| {
										for realm in [Realm::Menu, Realm::Client] {
											let (color, text) = match realm {
												Realm::Menu => (Color32::from_rgb(100, 200, 100), "Menu"),
												Realm::Client => (Color32::from_rgb(255, 165, 0), "Client"),
											};
											ui.scope(|ui| {
												ui.style_mut().visuals.override_text_color = Some(color);
												ui.selectable_value(&mut self.realm_state, realm, text);
											});
										}
									});
							});
						});
					});

					ui.separator();

					// Code editor
					let editor_height = ui.available_height() - 40.0;

					let (rect, _response) =
						ui.allocate_exact_size(Vec2::new(ui.available_width(), editor_height), egui::Sense::hover());

					ui.allocate_ui_at_rect(rect, |ui| {
						let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
							let mut layout_job =
								egui_extras::syntax_highlighting::highlight(ui.ctx(), &CodeTheme::dark(), string, "lua");
							layout_job.wrap.max_width = wrap_width;
							ui.fonts(|f| f.layout_job(layout_job))
						};

						ui.add_sized(
							[ui.available_width(), editor_height],
							TextEdit::multiline(&mut self.code)
								.font(egui::TextStyle::Monospace)
								.code_editor()
								.desired_width(f32::INFINITY)
								.layouter(&mut layouter),
						);
					});
				});
			});
	}

	fn show_settings_tab(&mut self, ui: &mut Ui) {
		ui.vertical_centered(|ui| {
			ui.add_space(50.0);
			ui.heading("Settings");
			ui.add_space(20.0);

			Frame::default()
				.fill(Color32::from_rgb(25, 25, 25))
				.stroke(Stroke::new(1.0, Color32::from_rgb(50, 50, 50)))
				.rounding(Rounding::same(8.0))
				.inner_margin(Margin::same(20.0))
				.show(ui, |ui| {
					ui.label("Settings panel - Coming soon!");
					ui.add_space(10.0);
					ui.label("• Theme customization");
					ui.label("• Hotkey configuration");
					ui.label("• Auto-connect settings");
					ui.label("• Font size preferences");
				});
		});
	}

	fn show_about_tab(&mut self, ui: &mut Ui) {
		ui.vertical_centered(|ui| {
			ui.add_space(50.0);
			ui.heading("About Autorun-ng");
			ui.add_space(20.0);

			Frame::default()
				.fill(Color32::from_rgb(25, 25, 25))
				.stroke(Stroke::new(1.0, Color32::from_rgb(50, 50, 50)))
				.rounding(Rounding::same(8.0))
				.inner_margin(Margin::same(20.0))
				.show(ui, |ui| {
					ui.label("A modern Lua executor and terminal for game automation");
					ui.add_space(10.0);
					ui.label(concat!("Version: ", env!("CARGO_PKG_VERSION")));
					ui.label("Built with Rust + egui");
					ui.add_space(15.0);

					if ui.link("GitHub Repository").clicked() {
						open_url("https://github.com/thevurv/Autorun-ng");
					}

					if ui.link("📚 Documentation").clicked() {
						// Handle link click
					}
				});
		});
	}

	fn show_status_bar(&mut self, ui: &mut Ui) {
		Frame::default()
			.fill(Color32::from_rgb(30, 30, 30))
			.stroke(Stroke::new(1.0, Color32::from_rgb(50, 50, 50)))
			.inner_margin(Margin::symmetric(12.0, 6.0))
			.show(ui, |ui| {
				ui.horizontal(|ui| {
					ui.label(format!(
						"v{} - {}",
						env!("CARGO_PKG_VERSION"),
						if cfg!(debug_assertions) { "Debug" } else { "Release" }
					));

					ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
						if ui.link("Discord").clicked() {
							open_url("https://discord.gg/cSC3ebaR3q");
						}

						ui.separator();

						if ui.link("GitHub").clicked() {
							open_url("https://github.com/thevurv/Autorun-ng");
						}
					});
				});
			});
	}

	fn execute_terminal_command(&mut self) {
		if self.terminal_input.is_empty() {
			return;
		}

		let mut context = CommandContext::new(Arc::clone(&self.log), &self.autorun);

		match self.command_registry.execute_command(&self.terminal_input, &mut context) {
			Ok(true) => {
				// Command was handled
			}
			Ok(false) => {
				// Not a recognized command
				context.write_error(&format!("Unknown command: '{}'", self.terminal_input));
			}
			Err(e) => {
				eprintln!("Command execution error: {}", e);
			}
		}

		self.terminal_input.clear();
	}

	fn execute_lua_code(&mut self) {
		if self.code.is_empty() {
			return;
		}

		if let Err(e) = self.autorun.run_code(self.realm_state, &self.code) {
			error!("Failed to execute code: {e}");
		}
	}

	fn render_ansi_text(&self, ui: &mut Ui, text: &str) {
		let segments = parse_ansi_text(text);

		ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

		// Group segments by lines to render them inline
		let mut current_job = LayoutJob::default();
		let font_id = FontId::new(11.0, FontFamily::Monospace);

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
					let color = segment.color.unwrap_or(Color32::from_rgb(220, 220, 220));
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
	}
}

impl eframe::App for App {
	fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
		let _ = self.autorun.detach();
	}

	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		// Only update autorun status periodically to avoid blocking UI
		if self.last_update.elapsed() >= UPDATE_INTERVAL {
			if !self.user_disconnected {
				self.autorun.update();
			}
			self.last_update = std::time::Instant::now();
		}

		egui::CentralPanel::default().show(ctx, |ui| {
			ui.vertical(|ui| {
				// Header
				self.show_header(ui);

				// Tab bar
				self.show_tab_bar(ui);

				let content_height = ui.available_height() - 50.0; // Reserve space for status bar
				ui.allocate_ui(Vec2::new(ui.available_width(), content_height), |ui| {
					ui.set_min_height(content_height);
					match self.active_tab {
						ActiveTab::Console => self.show_console_tab(ui),
						ActiveTab::Settings => self.show_settings_tab(ui),
						ActiveTab::About => self.show_about_tab(ui),
					}
				});

				// Status bar
				self.show_status_bar(ui);
			});
		});
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
			for ch in chars.by_ref() {
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

fn open_url(url: &str) {
	#[cfg(target_os = "windows")]
	{
		let _ = std::process::Command::new("cmd").args(["/C", "start", url]).spawn();
	}

	#[cfg(target_os = "macos")]
	{
		let _ = std::process::Command::new("open").arg(url).spawn();
	}

	#[cfg(target_os = "linux")]
	{
		let _ = std::process::Command::new("xdg-open").arg(url).spawn();
	}
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
