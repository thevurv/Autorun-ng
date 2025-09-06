use std::io::{Read, Seek};

struct Permissions {
	read: bool,
	write: bool,
	exec: bool,
	private: bool,
}

fn parse_perms(perms: &str) -> Permissions {
	Permissions {
		read: perms.chars().nth(0) == Some('r'),
		write: perms.chars().nth(1) == Some('w'),
		exec: perms.chars().nth(2) == Some('x'),
		private: perms.chars().nth(3) == Some('p'),
	}
}

pub fn scan(signature: &[Option<u8>]) -> Result<Option<usize>, std::io::Error> {
	let maps = std::fs::read_to_string("/proc/self/maps")?;
	let mut mem = std::fs::File::open("/proc/self/mem")?;

	for line in maps.lines() {
		let entries = line.split_whitespace().collect::<Vec<_>>();

		let raw_range = entries[0];
		let (start, end) = raw_range.split_once('-').expect("Malformed maps entry");
		let (start, end) = (
			usize::from_str_radix(start, 16).expect("Malformed start address"),
			usize::from_str_radix(end, 16).expect("Malformed end address"),
		);

		let size = end - start;
		let perms = parse_perms(entries[1]);
		let _offset = entries[2];
		let _dev = entries[3];
		let _inode = entries[4];
		let _path = entries.get(5); // Optional

		if !perms.read || !perms.exec {
			continue;
		}

		let mut buffer = vec![0u8; size];
		mem.seek(std::io::SeekFrom::Start(start as u64))?;

		if mem.read_exact(&mut buffer).is_ok() {
			// Search for signature in this buffer
			if let Some(offset) = find_signature(&buffer, signature) {
				return Ok(Some(start + offset));
			}
		}
	}

	Ok(None)
}

fn find_signature(haystack: &[u8], needle: &[Option<u8>]) -> Option<usize> {
	if needle.is_empty() {
		return Some(0);
	}

	for i in 0..=haystack.len().saturating_sub(needle.len()) {
		let mut matches = true;

		for (j, &pattern_byte) in needle.iter().enumerate() {
			if let Some(expected) = pattern_byte {
				if haystack[i + j] != expected {
					matches = false;
					break;
				}
			}
		}

		if matches {
			return Some(i);
		}
	}
	None
}
