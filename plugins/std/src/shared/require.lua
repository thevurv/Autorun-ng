local unpack = unpack

---@type table<lightuserdata, table<string, table | true>>
local cache = {}

function Autorun.require(path)
    cache[Autorun.PLUGIN] = cache[Autorun.PLUGIN] or {}
    local localCache = cache[Autorun.PLUGIN]

    if localCache[path] == true then
        error("Circular dependency detected for path: " .. path)
    elseif localCache[path] then
        return unpack(cache[path])
    end

    localCache[path] = true
    local outputs = { Autorun.include(path) }
    localCache[path] = outputs

    return unpack(outputs)
end
