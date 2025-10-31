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

_G.print("Is GetHostName authorized: " .. tostring(Autorun.isFunctionAuthorized(_G.GetHostName)))
_G.TestAuth = function()
    -- function which called us would be at stack level 2
    _G.print("Authed: " .. tostring(Autorun.isFunctionAuthorized(2)))
end

_G.print("Calling TestAuth from an authorized context:")
_G.TestAuth()
