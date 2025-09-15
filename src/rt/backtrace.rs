// todo, backtracing seems to be quite complicated.
//
// A temporary solution would be to just use backtrace-rs, the library used by
// `std` for backtraces:
// https://github.com/rust-lang/backtrace-rs
//
// On Unix backtrace-rs uses libunwind for backtraces:
// https://github.com/libunwind/libunwind
//
// libc also offers backtrace functions I could use (`man backtrace`).
