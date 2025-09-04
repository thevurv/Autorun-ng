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
	pub execute: Box<
		dyn Fn(&[String], &mut CommandContext<'_>, &CommandRegistry) -> Result<(), String>
			+ Send
			+ Sync,
	>,
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
	pub fn execute_command(
		&self,
		input: &str,
		context: &mut CommandContext<'_>,
	) -> Result<bool, String> {
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
						context.write_output(&format!(
							"\x1b[1m{}\x1b[0m - {}",
							command.name, command.description
						));
						context.write_output(&format!("  Usage: {}", command.usage));
						context.write_output("");
					}

					context.write_info("Type a command name to execute it.");
					context.write_info("Unknown commands will show an error message.");
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

		// Status command
		self.register_command(Command {
			name: "status".to_string(),
			description: "Show connection status and autorun information".to_string(),
			usage: "status".to_string(),
			execute: Box::new(|_args, context, _registry| {
				use crate::backend::AutorunStatus;

				context.write_info("=== Autorun Status ===");
				match context.autorun.status() {
					AutorunStatus::Connected => {
						context.write_success("Status: Connected to game");
					}
					AutorunStatus::Disconnected => {
						context.write_warning("Status: Disconnected from game");
					}
				}
				context.write_output("");
				Ok(())
			}),
		});
	}

	/// Example utility function for registering custom commands
	/// This demonstrates how to easily add new commands to the registry
	pub fn register_custom_commands(&mut self) {
		// Console command to send input to game
		self.register_command(Command {
			name: "console".to_string(),
			description: "Send a command to the game console".to_string(),
			usage: "console <command>".to_string(),
			execute: Box::new(|args, context, _registry| {
				if args.is_empty() {
					context.write_warning("Usage: console <command>");
				} else {
					let command = args.join(" ");
					if let Err(e) = context.autorun.print_to_game(&command) {
						context.write_error(&format!("Failed to send to game: {}", e));
					} else {
						context.write_success(&format!("Sent to game: {}", command));
					}
				}
				Ok(())
			}),
		});
		// Example: Echo command that repeats the input
		self.register_command(Command {
			name: "echo".to_string(),
			description: "Echo the provided text back to the terminal".to_string(),
			usage: "echo <text>".to_string(),
			execute: Box::new(|args, context, _registry| {
				if args.is_empty() {
					context.write_warning("Usage: echo <text>");
				} else {
					let message = args.join(" ");
					context.write_output(&format!("Echo: {}", message));
				}
				Ok(())
			}),
		});

		// Example: Time command that shows current time
		self.register_command(Command {
			name: "time".to_string(),
			description: "Display the current time".to_string(),
			usage: "time".to_string(),
			execute: Box::new(|_args, context, _registry| {
				use std::time::SystemTime;
				match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
					Ok(duration) => {
						let timestamp = duration.as_secs();
						context.write_info(&format!("Current Unix timestamp: {}", timestamp));
					}
					Err(_) => {
						context.write_error("Failed to get current time");
					}
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_command_registry_creation() {
		let registry = CommandRegistry::new();
		assert!(registry.commands.contains_key("help"));
		assert!(registry.commands.contains_key("clear"));
		assert!(registry.commands.contains_key("status"));
	}

	#[test]
	fn test_command_parsing() {
		let registry = CommandRegistry::new();

		// Test non-command input
		let log = Arc::new(RwLock::new(String::new()));
		let autorun = Autorun::default();
		let mut context = CommandContext::new(log, &autorun);

		let result = registry.execute_command("nonexistent_command", &mut context);
		assert!(result.is_ok());
		assert!(!result.unwrap()); // Should return false for non-commands
	}

	#[test]
	fn test_help_command() {
		let registry = CommandRegistry::new();
		let log = Arc::new(RwLock::new(String::new()));
		let autorun = Autorun::default();
		let mut context = CommandContext::new(log.clone(), &autorun);

		let result = registry.execute_command("help", &mut context);
		assert!(result.is_ok());
		assert!(result.unwrap()); // Should return true for commands

		// Check that output was written to log
		let log_content = log.read().unwrap();
		assert!(log_content.contains("Available commands"));
	}
}
