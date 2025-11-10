use crate::{IntoLua, LuaApi, LuaState};

pub trait LuaReturn {
	fn into_lua_return(self, lua: &LuaApi, state: *mut LuaState) -> i32;
}

impl LuaReturn for () {
	fn into_lua_return(self, _: &LuaApi, _: *mut LuaState) -> i32 {
		0
	}
}

impl<T: IntoLua> LuaReturn for T {
	fn into_lua_return(self, lua: &LuaApi, state: *mut LuaState) -> i32 {
		self.into_lua(lua, state);
		1
	}
}

#[repr(transparent)]
pub struct RawLuaReturn(pub i32);

impl LuaReturn for RawLuaReturn {
	fn into_lua_return(self, _lua: &LuaApi, _state: *mut LuaState) -> i32 {
		self.0
	}
}

// Macro to implement LuaReturn for tuples
macro_rules! impl_lua_return_tuple {
    ($($T:ident),+) => {
        impl<$($T: $crate::IntoLua),+> LuaReturn for ($($T,)+) {
            fn into_lua_return(self, lua: &LuaApi, state: *mut LuaState) -> i32 {
                #[allow(non_snake_case)]
                let ($($T,)+) = self;
                let mut count = 0;
                $(
                    $T.into_lua(lua, state);
                    count += 1;
                )+
                count
            }
        }
    };
}

impl_lua_return_tuple!(T1);
impl_lua_return_tuple!(T1, T2);
impl_lua_return_tuple!(T1, T2, T3);
impl_lua_return_tuple!(T1, T2, T3, T4);
impl_lua_return_tuple!(T1, T2, T3, T4, T5);
impl_lua_return_tuple!(T1, T2, T3, T4, T5, T6);
impl_lua_return_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_lua_return_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_lua_return_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_lua_return_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_lua_return_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_lua_return_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
