use std::path::{Path, PathBuf};

fn main() {
    let yoga = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/yoga");
    let mut sources = Vec::new();
    collect_cpp(&yoga.join("yoga"), &mut sources);
    cc::Build::new()
        .cpp(true)
        .std("c++20")
        // Rust and C++ exception models differ under Emscripten. Built without exceptions, Yoga
        // aborts on a fatal error instead of throwing.
        .flag("-fno-exceptions")
        // No fused multiply-add: WebAssembly has none, so fusing on native targets (Apple clang
        // does by default on arm64) rounds differently and boxes stop matching across platforms.
        .flag("-ffp-contract=off")
        .include(&yoga)
        .files(&sources)
        .warnings(false)
        .compile("yoga");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("emscripten") {
        // rustc drives emcc in C mode, which leaves libc++ out of the link.
        println!("cargo:rustc-link-arg=-sDEFAULT_TO_CXX");
    }
    println!("cargo:rerun-if-changed={}", yoga.display());
}

fn collect_cpp(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("vendor/yoga is missing") {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_cpp(&path, out);
        } else if path.extension().is_some_and(|e| e == "cpp") {
            out.push(path);
        }
    }
}
