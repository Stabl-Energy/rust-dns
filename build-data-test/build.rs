fn main() {
    build_data::set_SOURCE_DATE();
    build_data::set_SOURCE_TIME();
    build_data::set_SOURCE_TIMESTAMP();
    build_data::set_SOURCE_EPOCH_TIME();
    build_data::set_BUILD_DATE();
    build_data::set_BUILD_TIME();
    build_data::set_BUILD_TIMESTAMP();
    build_data::set_BUILD_EPOCH_TIME();
    build_data::set_BUILD_HOSTNAME();
    build_data::set_GIT_BRANCH();
    build_data::set_GIT_COMMIT();
    build_data::set_GIT_COMMIT_SHORT();
    build_data::set_GIT_DIRTY();
    build_data::set_RUSTC_VERSION();
    build_data::set_RUSTC_VERSION_SEMVER();
    build_data::set_RUST_CHANNEL();

    // Cargo sets some variables automatically:
    // - CARGO_PKG_VERSION
    // - CARGO_PKG_NAME
    // - CARGO_CRATE_NAME
    // - Many others:
    //   https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates

    // Speed up dev builds.
    build_data::no_debug_rebuilds();
}
