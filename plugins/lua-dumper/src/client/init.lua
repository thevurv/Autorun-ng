-- Store it once for those weird servers that change hostnames mid-session.
local hostName = string.match(_G.GetHostName(), "^([%w_%-][%w _%-']*)$") or "unknown_host"

Autorun.print(Autorun.color("green") .. "Started!" .. Autorun.color("reset"))

Autorun.on("loadbuffer", function(scriptName, scriptCode)
    if string.sub(scriptName, 1, 1) == "@" then
        scriptName = string.sub(scriptName, 2)
    end

    -- A little bit of extra sanitizing.
    local parentDir = string.match(scriptName, "^(.*)/") or "."

    Autorun.mkdir(hostName .. "/" .. parentDir)
    Autorun.writeAsync(hostName .. "/" .. scriptName, scriptCode)
end)

Autorun.print("Hi!")
Autorun.print("\n" .. debug.traceback())

_G.TryTheThing = function()
    jit.attach(function(proto)
        local isAuthorized = Autorun.isProtoAuthorized(proto)
        Autorun.print("JIT attach: " ..
            (isAuthorized and "authorized" or "unauthorized") .. " proto: " .. tostring(proto))
    end, "bc")

    -- make functions to trigger it

    Autorun.load("_G.print('Hi from test bc trigger!')", "test_bc_trigger.lua")()
end
