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
Autorun.print("Detour: " .. tostring(Autorun.detour))
testDetour = Autorun.detour(_G.game.GetIPAddress, function(orig, ...)
    Autorun.print("Autorun.detour is working! Calling original function...")
    _G.print("Hi from autorun detour! Returning original value.")
    local args = {...}
    _G.print("Arguments passed to game.GetIPAddress: ")
    _G.PrintTable(args)
    return orig()
end)

Autorun.print("Detour test: " .. tostring(testDetour))