---@class Autorun
---@field NAME string # Name of script running
---@field MODE "init" | "menu" | "hook" # Mode in which script is running
---@field CODE string # Source code of script
---@field CODE_LEN integer # Length of source code
---@field PATH string # Path to the currently running script, local to /autorun/. Shouldn't really be used (and definitely not modified.)
Autorun = {}

--- Prints to the Autorun console in the format of [Lua]: ...
---@param ... any # Values to print
function Autorun.print(...) end
