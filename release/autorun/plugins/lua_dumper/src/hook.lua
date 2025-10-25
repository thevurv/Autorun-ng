local scriptName = Autorun.NAME
if string.sub(scriptName, 1, 1) == "@" then
	scriptName = string.sub(scriptName, 2)
end

-- A little bit of extra sanitizing.
local hostName = string.match(GetHostName(), "^([%w_%-][%w _%-']*)$") or "unknown_host"
local parentDir = string.match(scriptName, "^(.*)/") or "."

Autorun.mkdir(hostName .. "/" .. parentDir)
Autorun.writeAsync(hostName .. "/" .. scriptName, Autorun.CODE)
