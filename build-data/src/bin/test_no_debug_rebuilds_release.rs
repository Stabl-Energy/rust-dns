#![forbid(unsafe_code)]
fn main() {
    std::env::set_var("PROFILE", "release");
    build_data::no_debug_rebuilds();
}
