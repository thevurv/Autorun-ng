type TheTargetFn = extern "C" fn(arg1: i64, arg2: i64, arg3: u64);
// type TheTargetFn = extern "C" fn() -> u8;

static THE_TARGET_FN_H: std::sync::OnceLock<retour::GenericDetour<TheTargetFn>> = std::sync::OnceLock::new();

extern "C" fn the_target_fn_h(a: i64, b: i64, c: u64) {
	// extern "C" fn the_target_fn_h() -> u8 {
	THE_TARGET_FN_H.get().unwrap().call(a, b, c);
	autorun_log::info!("Called with {a:x} {b:x} {c:x}");
}

pub fn init() -> anyhow::Result<()> {
	// let scan_result = autorun_scan::scan(autorun_scan::sig![0x8b, 0x88, 0x0c, 0x10, 0x00, 0x00, 0x85, 0xc9, 0x0f, 0x95])?;
	// let scan_result = autorun_scan::scan(autorun_scan::sig![
	// 	0x48, 0x8b, 0x05, ?, ?, ?, ?, 0x55, 0x48, 0x89, 0xe5, 0x5d, 0x48, 0x8b, 0x00
	// ])?;
	// Alternative scan for different signature
	let scan_result = autorun_scan::scan(autorun_scan::sig![
		0x55, 0x48, 0x89, 0xe5, 0x41, 0x55, 0x41, 0x54, 0x49, 0x89, 0xfc, 0x53, 0x48, 0x83, 0xec, ?, 0x48, 0x8b, 0x77, ?
	])?;
	if let Some(addr) = scan_result {
		let fn_start_addr = addr - 0;
		// let fn_start_addr = addr - 33;
		// let fn_start_addr = addr - 0;

		// Verify function prelude
		let expected_bytes = [0x55, 0x48, 0x89, 0xe5, 0x41];
		// let expected_bytes = &[0x48, 0x8b, 0x05, 0x01];
		unsafe {
			let actual_bytes = std::slice::from_raw_parts(fn_start_addr as *const u8, expected_bytes.len());
			if actual_bytes != expected_bytes {
				autorun_log::error!(
					"Function start verification failed at 0x{:x}. Expected: {:02x?}, Found: {:02x?}",
					fn_start_addr,
					expected_bytes,
					actual_bytes
				);

				return Err(anyhow::anyhow!("Function start verification failed"));
			}

			autorun_log::info!("Function start verification passed at 0x{:x}", fn_start_addr);
		}
		let target_fn: TheTargetFn = unsafe { std::mem::transmute(fn_start_addr as *const ()) };

		let detour = unsafe {
			let detour = retour::GenericDetour::new(target_fn, the_target_fn_h)?;
			detour.enable()?;
			detour
		};

		THE_TARGET_FN_H.set(detour).unwrap();
	} else {
		autorun_log::warn!("Function not found");
	}

	Ok(())
}
