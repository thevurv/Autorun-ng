local formats = {}

do
    local colors = {
        black = 30,
        red = 31,
        green = 32,
        yellow = 33,
        blue = 34,
        magenta = 35,
        cyan = 36,
        white = 37,
        reset = 0
    }

    function formats.ansi(rgbInput, rawInput)
        if colors[rawInput] then
            return "\27[" .. colors[rawInput] .. "m"
        end

        return string.format("\27[38;2;%d;%d;%dm", rgbInput.r, rgbInput.g, rgbInput.b)
    end

    function formats.hex(rgbInput, rawInput)
        return string.format("#%02X%02X%02X", rgbInput.r, rgbInput.g, rgbInput.b)
    end

    function formats.number(rgbInput, rawInput)
        return bit.bor(bit.lshift(rgbInput.r, 16), bit.lshift(rgbInput.g, 8), rgbInput.b)
    end

    formats["{rgb}"] = function(rgbInput, rawInput)
        return { r = rgbInput.r, g = rgbInput.g, b = rgbInput.b }
    end

    formats["[rgb]"] = function(rgbInput, rawInput)
        return { rgbInput.r, rgbInput.g, rgbInput.b }
    end
end

local colorToRgb = {
    black = { r = 0, g = 0, b = 0 },
    red = { r = 255, g = 0, b = 0 },
    orange = { r = 255, g = 165, b = 0 },
    green = { r = 0, g = 255, b = 0 },
    yellow = { r = 255, g = 255, b = 0 },
    blue = { r = 0, g = 0, b = 255 },
    magenta = { r = 255, g = 0, b = 255 },
    cyan = { r = 0, g = 255, b = 255 },
    white = { r = 255, g = 255, b = 255 },
    reset = { r = 255, g = 255, b = 255 }
}

function Autorun.color(rawInput, outputFormat)
    outputFormat = outputFormat or "ansi"

    assert(formats[outputFormat], "Unsupported color format: " .. tostring(outputFormat))

    local rgbInput
    if type(rawInput) == "string" then
        if string.sub(rawInput, 1, 1) == "#" then
            local r = tonumber(string.sub(rawInput, 2, 3), 16)
            local g = tonumber(string.sub(rawInput, 4, 5), 16)
            local b = tonumber(string.sub(rawInput, 6, 7), 16)

            rgbInput = { r = r, g = g, b = b }
        elseif colorToRgb[rawInput] then
            rgbInput = colorToRgb[rawInput]
        else
            error("Unsupported color input: " .. tostring(rawInput))
        end
    elseif type(rawInput) == "number" then
        rgbInput = {
            r = bit.rshift(bit.band(rawInput, 0xFF0000), 16),
            g = bit.rshift(bit.band(rawInput, 0x00FF00), 8),
            b = bit.band(rawInput, 0x0000FF)
        }
    elseif type(rawInput) == "table" then
        if type(rawInput[1]) == "number" and type(rawInput[2]) == "number" and type(rawInput[3]) == "number" then
            rgbInput = { r = rawInput[1], g = rawInput[2], b = rawInput[3] }
        elseif type(rawInput.r) == "number" and type(rawInput.g) == "number" and type(rawInput.b) == "number" then
            rgbInput = { r = rawInput.r, g = rawInput.g, b = rawInput.b }
        else
            error("Color table must be of format {r, g, b} or {r = r, g = g, b = b}")
        end
    else
        error("Unsupported color input type: " .. type(rawInput))
    end

    return formats[outputFormat](rgbInput, rawInput)
end
