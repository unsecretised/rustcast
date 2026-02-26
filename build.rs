fn main() {
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-search=framework=/System/Library/PrivateFrameworks");
        println!("cargo:rustc-link-lib=framework=IOKit");
        println!("cargo:rustc-link-lib=framework=MultitouchSupport");
    }
}
