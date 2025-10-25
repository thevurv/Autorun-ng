-- Define our own builtins to avoid detectable _G accesses

string = {
    sub = _G.string.sub,
    upper = _G.string.upper,
    lower = _G.string.lower,
    find = _G.string.find,
    gsub = _G.string.gsub,
    gmatch = _G.string.gmatch,
    match = _G.string.match,
    rep = _G.string.rep,
    reverse = _G.string.reverse,
    len = _G.string.len,
    byte = _G.string.byte,
    char = _G.string.char,
    format = _G.string.format,
}

GetHostName = _G.GetHostName
