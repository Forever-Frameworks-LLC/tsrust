fn main() {
    // Generated workspaces share a Cargo target and use identical relative source paths.
    println!("cargo:rerun-if-env-changed=TSRUST_GPUI_PATCH_REVISION");
    println!("cargo:rerun-if-changed=build.rs");
}
