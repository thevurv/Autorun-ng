local unpack = unpack
local includeCache = {}

function Autorun.require(path)
    if includeCache[path] then
        return unpack(includeCache[path])
    end

    local outputs = { Autorun.include(path) }
    includeCache[path] = outputs

    return unpack(outputs)
end
