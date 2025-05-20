use std::{env, path::PathBuf};

fn main() {
    let mut build = cc::Build::new();
    #[cfg(not(feature = "simd"))]
    build.define("QOIR_CONFIG__DISABLE_SIMD", None);

    #[cfg(feature = "large_luts")]
    build.define("QOIR_CONFIG__DISABLE_LARGE_LOOK_UP_TABLES", None);

    build
        .file("src/qoir.c")
        .include("vendor/qoir/src")
        .compile("qoir");

    let bindings = bindgen::Builder::default()
        .header("vendor/qoir/src/qoir.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    bindings
        .write_to_file(out_path.join("qoir_bindings.rs"))
        .expect("Couldn't write bindings!");
}
