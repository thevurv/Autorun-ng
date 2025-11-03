-- These are all core protections against detection and exploitation attempts.

local function wrapFastFunctionInSafeCall(fn)
    return Autorun.copyFastFunction(fn, function(...)
        -- This is an awful hack, but TCO (tail-call optimization) causes issues with function authorization,
        -- and basically moves all of our Autorun code to the actual Lua code which called a protected function,
        -- and it will fail authorization checks and bug out. This forces a call frame to be generated each time,
        -- which passes through our authorization checks correctly.
        local results = { Autorun.safeCall(fn, ...) }
        return unpack(results)
    end)
end

_G.getfenv = wrapFastFunctionInSafeCall(_G.getfenv)
_G.debug.getinfo = wrapFastFunctionInSafeCall(_G.debug.getinfo)
_G.debug.getlocal = wrapFastFunctionInSafeCall(_G.debug.getlocal)
_G.debug.traceback = wrapFastFunctionInSafeCall(_G.debug.traceback)
_G.setfenv = wrapFastFunctionInSafeCall(_G.setfenv)
_G.error = wrapFastFunctionInSafeCall(_G.error)
