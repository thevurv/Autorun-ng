local tostring = tostring
local CompileString = CompileString
local error = error

--- Reads and executes a Lua file from the given path.
--- @param target_path string
--- @return any
function Autorun.include(target_path)
    local content = Autorun.read(target_path)
    if not content then
        error("Failed to read file for include '" .. target_path .. "'")
    end

    local ok, err = CompileString(content)
    if not ok then
        error("Failed to compile file " .. target_path .. ": " .. tostring(err))
    end

    return ok()
end
