local tostring = tostring
local CompileString = CompileString
local error = error

function Autorun.include(path)
    local content = Autorun.read(path)
    if not content then
        error("Failed to read file for include '" .. path .. "'")
    end

    local ok, err = CompileString(content)
    if not ok then
        error("Failed to compile file " .. path .. ": " .. tostring(err))
    end

    return ok()
end
