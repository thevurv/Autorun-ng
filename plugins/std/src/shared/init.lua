local tostring = _G.tostring
local error = _G.error
local setfenv = _G.setfenv
local getfenv = _G.getfenv

local autorunEnv = getfenv(1)

function Autorun.include(path)
    local content = Autorun.read("src/" .. path)
    if not content then
        error("Failed to read file for include '" .. path .. "'")
    end

    local ok, err = Autorun.load(content)
    if not ok then
        error("Failed to compile file " .. path .. ": " .. tostring(err))
    end

    setfenv(ok, autorunEnv)

    return ok()
end

Autorun.include("shared/builtins.lua")
Autorun.include("shared/require.lua")
Autorun.include("shared/event.lua")
