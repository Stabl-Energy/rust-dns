#![forbid(unsafe_code)]
fn main() {
    std::env::set_var("PROFILE", "debug");
    build_data::no_debug_rebuilds();
}
