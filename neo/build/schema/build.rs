#[cfg(feature = "syntex")]
fn main() {
    extern crate syntex;
    extern crate serde_codegen;

    use std::env;
    use std::path::Path;

    let mut registry = syntex::Registry::new();
    serde_codegen::register(&mut registry);

    let src = Path::new("src/lib.rs");
    let dst = Path::new(&env::var("OUT_DIR").unwrap()).join("lib.rs");

    registry.expand("lib", &src, &dst).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/mod.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
}

#[cfg(not(feature = "syntex"))]
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
}
