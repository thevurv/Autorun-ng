pub struct Module {
	pub base_address: usize,
	pub size: usize,
	pub name: String,
}

fn get_module_list() -> Result<Vec<Module>, std::io::Error> {
	use windows::Win32::Foundation::HANDLE;
	use windows::Win32::System::Diagnostics::ToolHelp::{
		CreateToolhelp32Snapshot, MODULEENTRY32, Module32First, Module32Next, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32,
	};

	let snapshot: HANDLE = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, 0).unwrap() };
	if snapshot.is_invalid() {
		Err(std::io::Error::last_os_error())
	} else {
		let mut modules = Vec::new();
		let mut module_entry = MODULEENTRY32 {
			dwSize: std::mem::size_of::<MODULEENTRY32>() as u32,
			..Default::default()
		};

		let mut success = unsafe { Module32First(snapshot, &mut module_entry).is_ok() };
		while success {
			let name = String::from_utf8(
				module_entry
					.szModule
					.iter()
					.take_while(|&&c| c != 0)
					.map(|&c| c as u8)
					.collect(),
			)
			.unwrap_or_default();

			modules.push(Module {
				base_address: module_entry.modBaseAddr as usize,
				size: module_entry.modBaseSize as usize,
				name,
			});

			success = unsafe { Module32Next(snapshot, &mut module_entry).is_ok() };
		}

		Ok(modules)
	}
}

fn find_signature(haystack: &[u8], needle: &[Option<u8>]) -> Option<usize> {
	if needle.is_empty() {
		return Some(0);
	}

	for i in 0..=(haystack.len() - needle.len()) {
		let mut found = true;
		for (j, &byte_opt) in needle.iter().enumerate() {
			if let Some(byte) = byte_opt {
				if haystack[i + j] != byte {
					found = false;
					break;
				}
			}
		}
		if found {
			return Some(i);
		}
	}

	None
}

pub fn scan(signature: &[Option<u8>], target_module: Option<&str>) -> Result<Option<usize>, std::io::Error> {
	let modules = get_module_list()?;
	for module in modules {
		// If no target module specified, scan all modules
		let target_module = target_module.unwrap_or(&module.name);
		if module.name.eq_ignore_ascii_case(target_module) {
			unsafe {
				let module_bytes = std::slice::from_raw_parts(module.base_address as *const u8, module.size);
				if let Some(offset) = find_signature(module_bytes, signature) {
					return Ok(Some(module.base_address + offset));
				}
			}
		}
	}

	Ok(None)
}
