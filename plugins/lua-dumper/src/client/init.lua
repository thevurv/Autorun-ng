-- Store it once for those weird servers that change hostnames mid-session.
local hostName = string.match(_G.GetHostName(), "^([%w_%-][%w _%-']*)$") or "unknown_host"

Autorun.print(Autorun.color("green") .. "Started!" .. Autorun.color("reset"))

Autorun.on("loadbuffer", function(scriptName, scriptCode, scriptMode)
    if string.find(scriptName, "my secret file") then
        return "print('67')"
    end

    if string.sub(scriptName, 1, 1) == "@" then
        scriptName = string.sub(scriptName, 2)
    end

    -- A little bit of extra sanitizing.
    local parentDir = string.match(scriptName, "^(.*)/") or "."

    Autorun.mkdir(hostName .. "/" .. parentDir)
    Autorun.writeAsync(hostName .. "/" .. scriptName, scriptCode)
end)
