use std::{
	collections::HashMap,
	sync::{Arc, RwLock},
};

use crate::backend::Autorun;

/// Represents a command that can be executed in the terminal
pub struct Command {
	pub name: String,
	pub description: String,
	pub usage: String,
	pub execute: Box<dyn Fn(&[String], &mut CommandContext<'_>, &CommandRegistry) -> Result<(), String> + Send + Sync>,
}

/// Context passed to command execution functions
pub struct CommandContext<'a> {
	pub log: Arc<RwLock<String>>,
	pub autorun: &'a Autorun,
}

impl<'a> CommandContext<'a> {
	pub fn new(log: Arc<RwLock<String>>, autorun: &'a Autorun) -> Self {
		Self { log, autorun }
	}

	/// Write output to the terminal log
	pub fn write_output(&self, message: &str) {
		if let Ok(mut log) = self.log.write() {
			log.push_str(message);
			if !message.ends_with('\n') {
				log.push('\n');
			}
		}
	}

	/// Write error output to the terminal log (in red)
	pub fn write_error(&self, message: &str) {
		if let Ok(mut log) = self.log.write() {
			log.push_str(&format!("\x1b[31mError: {}\x1b[0m", message));
			if !message.ends_with('\n') {
				log.push('\n');
			}
		}
	}

	/// Write success output to the terminal log (in green)
	pub fn write_success(&self, message: &str) {
		if let Ok(mut log) = self.log.write() {
			log.push_str(&format!("\x1b[32m{}\x1b[0m", message));
			if !message.ends_with('\n') {
				log.push('\n');
			}
		}
	}

	/// Write info output to the terminal log (in cyan)
	pub fn write_info(&self, message: &str) {
		if let Ok(mut log) = self.log.write() {
			log.push_str(&format!("\x1b[36m{}\x1b[0m", message));
			if !message.ends_with('\n') {
				log.push('\n');
			}
		}
	}

	/// Write warning output to the terminal log (in yellow)
	pub fn write_warning(&self, message: &str) {
		if let Ok(mut log) = self.log.write() {
			log.push_str(&format!("\x1b[33m{}\x1b[0m", message));
			if !message.ends_with('\n') {
				log.push('\n');
			}
		}
	}
}

/// Registry for managing commands
pub struct CommandRegistry {
	commands: HashMap<String, Command>,
}

impl CommandRegistry {
	pub fn new() -> Self {
		let mut registry = Self {
			commands: HashMap::new(),
		};

		// Register built-in commands
		registry.register_builtin_commands();
		registry
	}

	/// Register a new command
	pub fn register_command(&mut self, command: Command) {
		self.commands.insert(command.name.clone(), command);
	}

	/// Execute a command with the given input
	pub fn execute_command(&self, input: &str, context: &mut CommandContext<'_>) -> Result<bool, String> {
		let parts: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();

		if parts.is_empty() {
			return Ok(false); // Empty input, let it pass through to game
		}

		let command_name = &parts[0];
		let args = &parts[1..];

		if let Some(command) = self.commands.get(command_name) {
			match (command.execute)(args, context, self) {
				Ok(()) => Ok(true),
				Err(e) => {
					context.write_error(&format!("Command '{}' failed: {}", command_name, e));
					Ok(true)
				}
			}
		} else {
			// Not a recognized command, let it pass through to game
			Ok(false)
		}
	}

	/// Get all registered commands
	pub fn get_commands(&self) -> &HashMap<String, Command> {
		&self.commands
	}

	/// Register built-in commands
	fn register_builtin_commands(&mut self) {
		// Help command
		self.register_command(Command {
			name: "help".to_string(),
			description: "Display available commands and their usage".to_string(),
			usage: "help [command]".to_string(),
			execute: Box::new(|args, context, registry| {
				if args.is_empty() {
					// Show all commands
					context.write_info("Available commands:");
					context.write_output("");

					// Get all commands and sort them alphabetically
					let mut commands: Vec<_> = registry.get_commands().values().collect();
					commands.sort_by(|a, b| a.name.cmp(&b.name));

					// Display each command with its description
					for command in commands {
						context.write_output(&format!("\x1b[1m{}\x1b[0m - {}", command.name, command.description));
						context.write_output(&format!("  Usage: {}", command.usage));
						context.write_output("");
					}
				} else {
					// Show help for specific command
					let command_name = &args[0];
					if let Some(command) = registry.get_commands().get(command_name) {
						context.write_info(&format!("Help for command '{}':", command_name));
						context.write_output("");
						context.write_output(&format!("Description: {}", command.description));
						context.write_output(&format!("Usage: {}", command.usage));
					} else {
						context.write_error(&format!("Unknown command: '{}'", command_name));
					}
				}
				Ok(())
			}),
		});

		// Clear command
		self.register_command(Command {
			name: "clear".to_string(),
			description: "Clear the terminal output".to_string(),
			usage: "clear".to_string(),
			execute: Box::new(|_args, context, _registry| {
				if let Ok(mut log) = context.log.write() {
					log.clear();
				}
				Ok(())
			}),
		});
	}
}

impl Default for CommandRegistry {
	fn default() -> Self {
		Self::new()
	}
}
