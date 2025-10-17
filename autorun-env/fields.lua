---@class Autorun
---@field NAME string # Name of script running
---@field MODE "init" | "menu" | "hook" # Mode in which script is running
---@field CODE string # Source code of script
---@field CODE_LEN integer # Length of source code
---@field DIR userdata # Path to directory containing script. Don't delete this or you'll brick your plugin
Autorun = Autorun

--- Prints to the Autorun console in the format of [Lua]: ...
--- @param ... any # Values to print
function Autorun.print(...) end
