local scriptName = Autorun.NAME
if string.sub(scriptName, 1, 1) == "@" then
	scriptName = string.sub(scriptName, 2)
end

local parentDir = string.match(scriptName, "^(.*)/")
if parentDir then
    Autorun.mkdir(parentDir)
end

Autorun.writeAsync(scriptName, Autorun.CODE)
