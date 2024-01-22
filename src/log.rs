#[cfg(debug_assertions)]
#[allow(unused)]
pub enum Log {
    General,
    Audio,
}

#[cfg(debug_assertions)]
impl ToString for Log {
    fn to_string(&self) -> String {
        use Log::*;
        match self {
            General => "general",
            Audio => "audio",
        }
        .to_string()
    }
}

//TODO: use a mutex probably
#[cfg(debug_assertions)]
pub static mut LOG: String = String::new();

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! log {
    ($type:ident, $lit:literal $(, $i:expr)* $(,)?) => {
        let out = {
            use $crate::log::Log::*;
            format!("<{}> {}\n", $type.to_string(), format!($lit, $($i,)?))
        };

        unsafe {
            use $crate::log::LOG;
            LOG += &out;
        }
    };
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! log {
    ($type:ident, $lit:literal $(, $i:expr)* $(,)?) => {};
}
