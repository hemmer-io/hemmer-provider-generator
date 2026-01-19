fn main() {
    // Link Windows system libraries required by libgit2-sys
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=advapi32");
    }
}
