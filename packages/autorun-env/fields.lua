--- @class Autorun
--- @field PLUGIN userdata # Currently running plugin pointer
--- @field VERSION string # Version of Autorun-ng using semver format
Autorun = Autorun

--- Prints to the Autorun console in the format of [Lua]: ...
--- @param ... any # Values to print
function Autorun.print(...) end

--- Reads a path relative to the active plugin's directory.
--- @param path string
--- @return string? # Contents of the file
function Autorun.read(path) end

--- Writes to a path relative to the active plugin's directory.
--- @param path string
--- @param content string
function Autorun.write(path, content) end

--- **ASYNCHRONOUSLY** Writes to a path relative to the active plugin's directory.
--- This is important to avoid blocking the main thread on large writes to avoid detection.
--- @param path string
--- @param content string
function Autorun.writeAsync(path, content) end

--- Reads and executes a Lua file from the given path.
--- This doesn't do any caching.
--- @param path string
--- @return any
function Autorun.include(path) end

--- Reads and executes a Lua file from the given path.
--- This caches the output of the include so subsequent calls return the same values.
--- @param path string
--- @return any
function Autorun.require(path) end

--- Makes a directory recursively
--- @param path string
--- @return boolean # Whether the directory was created successfully, false if exists
function Autorun.mkdir(path) end
