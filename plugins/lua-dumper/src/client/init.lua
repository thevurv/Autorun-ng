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


local orig2 = _G.error
_G.error = Autorun.copyFastFunction(orig2, function(message, level)
    return Autorun.safeCall(orig2, message, level)
end)

-- Purposely vulnerable Autorun detour that might be exploited by malicious ACs:
-- tostring can be specified by any table, so if an AC passes a table with a malicious tostring,
-- it could execute arbitrary code in the context of this plugin.

detour = Autorun.detour(_G.render.Capture, function(orig, tbl)
    _G.print("AUTORUN: Detoured render.Capture called with argument: " .. tostring(tbl)) -- Vulnerable tostring call
    return "poop"
end)
