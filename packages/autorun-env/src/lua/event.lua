Autorun.EVENTS = {}

---@type table<string, integer>
local EventCounters = {}

---@class EventHandler
---@field event string
---@field id integer
local EventHandler = {}
EventHandler.__index = EventHandler

function EventHandler:remove()
    if Autorun.EVENTS[self.event] and Autorun.EVENTS[self.event][self.id] then
        Autorun.EVENTS[self.event][self.id] = nil
    end
end

---@overload fun(eventName: "loadbuffer", callback: fun(name: string, content: string, mode: string): boolean | nil | string): EventHandler
function Autorun.on(eventName, callback)
    Autorun.EVENTS[eventName] = Autorun.EVENTS[eventName] or {}

    local idx = (EventCounters[eventName] or 0) + 1
    Autorun.EVENTS[eventName][idx] = callback
    EventCounters[eventName] = idx

    return setmetatable({ event = eventName, id = idx }, EventHandler)
end

local currentlyTriggering = {}

---@overload fun(eventName: "loadbuffer", a: string, b: string, c: string): boolean | nil | string
function Autorun.trigger(eventName, a, b, c, d, e, f)
    if not Autorun.EVENTS[eventName] then return end

    assert(not currentlyTriggering[eventName], "Recursive event triggering detected for event: " .. eventName)
    currentlyTriggering[eventName] = true

    local success, err = pcall(function()
        for _, callback in pairs(Autorun.EVENTS[eventName]) do
            local result = callback(a, b, c, d, e, f)

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
