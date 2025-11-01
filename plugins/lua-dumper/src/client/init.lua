-- Store it once for those weird servers that change hostnames mid-session.
local hostName = string.match(_G.GetHostName(), "^([%w_%-][%w _%-']*)$") or "unknown_host"

Autorun.print("Started Lua dumping plugin.")

Autorun.on("loadbuffer", function(scriptName, scriptCode)
    if string.sub(scriptName, 1, 1) == "@" then
        scriptName = string.sub(scriptName, 2)
    end

    -- A little bit of extra sanitizing.
    local parentDir = string.match(scriptName, "^(.*)/") or "."

    Autorun.mkdir(hostName .. "/" .. parentDir)
    Autorun.writeAsync(hostName .. "/" .. scriptName, scriptCode)
end)

local orig = _G.debug.traceback
_G.debug.traceback = Autorun.copyFastFunction(orig, function(override)
    if override then
        return orig()
    end

    return Autorun.safeCall(orig)
end)


local orig2 = _G.getfenv
_G.origGetfenv = orig2
_G.getfenv = Autorun.copyFastFunction(orig2, function(func)
    return Autorun.safeCall(orig2, func)
end)

local orig3 = _G.error
_G.origError = orig3

_G.error = Autorun.copyFastFunction(orig3, function(message, level)
    return Autorun.safeCall(orig3, message, level)
end)

_G.RunInAutorun = function(cb)
    _G.print("[Autorun] Running function in Autorun context...")
    local result = cb()
    _G.print("[Autorun] Function completed.")
    return result
end
