-- These are all core protections against detection and exploitation attempts.

local function wrapFastFunctionInSafeCall(fn)
    return Autorun.copyFastFunction(fn, function(...)
        return Autorun.safeCall(fn, ...)
    end)
end

_G.getfenv = wrapFastFunctionInSafeCall(_G.getfenv)
_G.debug.getinfo = wrapFastFunctionInSafeCall(_G.debug.getinfo)
_G.debug.getlocal = wrapFastFunctionInSafeCall(_G.debug.getlocal)
_G.debug.traceback = wrapFastFunctionInSafeCall(_G.debug.traceback)
_G.setfenv = wrapFastFunctionInSafeCall(_G.setfenv)
_G.error = wrapFastFunctionInSafeCall(_G.error)
