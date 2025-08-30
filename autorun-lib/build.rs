fn main() {
	// Add search path to the project root where lua_shared_client.so is located
	println!("cargo:rustc-link-search=native=../");

	// Link to the library (expects lua_shared_client.so, not liblua_shared_client.so)
	println!("cargo:rustc-link-lib=dylib=lua_shared_client");

	// Embed a runtime path (rpath) so the dynamic linker looks in the project root
	println!("cargo:rustc-link-arg=-Wl,-rpath,../");
}
