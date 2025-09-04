#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        eprintln!("\x1b[104m\x1b[97m\x1b[1m INFO \x1b[0m: {}", format_args!($($arg)*));
    }}
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        eprintln!("\x1b[103m\x1b[30m\x1b[1m WARN \x1b[0m: {}", format_args!($($arg)*));
    }}
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        eprintln!("\x1b[101m\x1b[97m\x1b[1m ERROR \x1b[0m: {}", format_args!($($arg)*));
    }}
}
