fn main(){
    let target = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    if target == "macos" {
        println!("cargo:rustc-link-lib=framework=AppKit");
    }
}
