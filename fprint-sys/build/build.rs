use std::env;
use std::path::PathBuf;

fn main() {
    let mut build_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let lib = pkg_config::Config::new().probe("libfprint").unwrap();

    for path in lib.include_paths.iter() {
        println!("cargo:include={}", path.to_str().unwrap());
    }

    let bindgen = bindgen::Builder::default().header("build/wrapper.h");

    let bindings = bindgen
        .generate_comments(true)
        .blacklist_type("max_align_t")
        .blacklist_type("__fsid_t")
        .generate()
        .unwrap();
    build_path.push("fprint.rs");
    let _ = bindings.write_to_file(build_path);
}
