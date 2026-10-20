fn main() {
    println!(
        "cargo:rustc-env=MM_TARGET={}",
        std::env::var("TARGET").unwrap()
    );
}
