use std::env;
use std::path::PathBuf;

fn main() {
    let mut build_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let lib = pkg_config::Config::new()
        .print_system_libs(false)
        .probe("libfprint")
        .unwrap();

    lib.include_paths.iter().for_each(|path| {
        println!("cargo:include={}", path.to_str().unwrap());
    });

    let bindgen = bindgen::Builder::default().header("stddef.h");
    let bindgen =
        lib.include_paths
            .iter()
            .zip(lib.libs.iter())
            .fold(bindgen, |bindgen, (include, lib)| {
                let mut include = include.clone();
                let lib_name = lib.clone() + ".h";
                include.push(lib_name);

                bindgen.header(include.to_str().unwrap())
            });

    let bindings = bindgen
        .generate_comments(true)
        .blacklist_type("max_align_t")
        .blacklist_type("__fsid_t")
        .generate()
        .unwrap();
    build_path.push("fprint.rs");
    let _ = bindings.write_to_file(build_path);
}
