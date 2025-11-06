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

table = {
    insert = _G.table.insert,
    remove = _G.table.remove,
    sort = _G.table.sort,
    concat = _G.table.concat
}

math = {
    abs = _G.math.abs,
    acos = _G.math.acos,
    asin = _G.math.asin,
    atan = _G.math.atan,
    atan2 = _G.math.atan2,
    ceil = _G.math.ceil,
    cos = _G.math.cos,
    cosh = _G.math.cosh,
    deg = _G.math.deg,
    exp = _G.math.exp,
    floor = _G.math.floor,
    fmod = _G.math.fmod,
    frexp = _G.math.frexp,
    huge = _G.math.huge,
    ldexp = _G.math.ldexp,
    log = _G.math.log,
    log10 = _G.math.log10,
    max = _G.math.max,
    min = _G.math.min,
    modf = _G.math.modf,
    pi = _G.math.pi,
    pow = _G.math.pow,
    rad = _G.math.rad,
    random = _G.math.random,
    randomseed = _G.math.randomseed,
    sin = _G.math.sin,
    sinh = _G.math.sinh,
    sqrt = _G.math.sqrt,
    tan = _G.math.tan,
    tanh = _G.math.tanh,
}

os = {
    clock = _G.os.clock,
    date = _G.os.date,
    difftime = _G.os.difftime,
    time = _G.os.time
}

debug = {
    debug = _G.debug.debug,
    gethook = _G.debug.gethook,
    getinfo = _G.debug.getinfo,
    getlocal = _G.debug.getlocal,
    getmetatable = _G.debug.getmetatable,
    getupvalue = _G.debug.getupvalue,
    getfenv = _G.debug.getfenv,
    sethook = _G.debug.sethook,
    setmetatable = _G.debug.setmetatable,
    setfenv = _G.debug.setfenv,
    traceback = _G.debug.traceback,
}

coroutine = {
    create = _G.coroutine.create,
    isyieldable = _G.coroutine.isyieldable,
    resume = _G.coroutine.resume,
    running = _G.coroutine.running,
    status = _G.coroutine.status,
    wrap = _G.coroutine.wrap,
    yield = _G.coroutine.yield,
}

bit = {
    band = _G.bit.band,
    bor = _G.bit.bor,
    bxor = _G.bit.bxor,
    bnot = _G.bit.bnot,
    lshift = _G.bit.lshift,
    rshift = _G.bit.rshift,
    rol = _G.bit.rol,
    ror = _G.bit.ror,
}

jit = {
    attach = _G.jit.attach,
    flush = _G.jit.flush,
    on = _G.jit.on,
    off = _G.jit.off,
    status = _G.jit.status,
    version = _G.jit.version,
}

assert = _G.assert
collectgarbage = _G.collectgarbage
dofile = _G.dofile
error = _G.error
getmetatable = _G.getmetatable
ipairs = _G.ipairs
next = _G.next
pairs = _G.pairs
pcall = _G.pcall
print = _G.print
rawequal = _G.rawequal
rawget = _G.rawget
rawset = _G.rawset
select = _G.select
setmetatable = _G.setmetatable
tonumber = _G.tonumber
tostring = _G.tostring
type = _G.type
xpcall = _G.xpcall
_VERSION = _G._VERSION
getfenv = _G.getfenv
setfenv = _G.setfenv
unpack = _G.unpack
