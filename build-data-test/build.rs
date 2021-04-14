use std::env;
use std::path::Path;

fn main() {
    build_data_writer::no_debug_rebuilds();
    build_data_writer::write(&Path::new(&env::var_os("OUT_DIR").unwrap()).join("build-data.txt"))
        .unwrap();
}
