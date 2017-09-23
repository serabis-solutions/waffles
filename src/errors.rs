// Create the Error, ErrorKind, ResultExt, and Result types
error_chain! {
    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error) #[cfg(unix)];
    }
}
