---@type table<string, function[]>
local events = {}

---@type table<string, integer>
local eventCounters = {}

function Autorun.onRemote(eventName, callback)
    local currentPlugin = Autorun.PLUGIN
    events[eventName] = events[eventName] or {}

    local idx = (eventCounters[eventName] or 0) + 1

    events[eventName][idx] = function(v)
        local previousPlugin = Autorun.PLUGIN

        Autorun.PLUGIN = currentPlugin
        local fnReturn = callback(v)
        Autorun.PLUGIN = previousPlugin

        return fnReturn
    end

    eventCounters[eventName] = idx
end

---@type table<string, boolean>
local currentlyTriggering = {}

function Autorun.runRemoteCallbacks(eventName, v)
    if not events[eventName] then return end

    assert(not currentlyTriggering[eventName], "Recursive remote event triggering detected for event: " .. eventName)
    currentlyTriggering[eventName] = true

    local success, err = pcall(function()
        for _, callback in ipairs(events[eventName]) do
            local result = callback(v)

            if result ~= nil then
                currentlyTriggering[eventName] = nil
                return result
            end
        end
    end)

    currentlyTriggering[eventName] = nil

    if not success then
        error(err, 2)
    end
end
