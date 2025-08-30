fn build_rc() {
    println!("cargo::rerun-if-env-changed=build.rc");
    println!("cargo::rerun-if-env-changed=auto-accept.manifest");
    #[cfg(not(debug_assertions))]
    embed_resource::compile("build.rc", embed_resource::NONE)
        .manifest_required()
        .unwrap();
}

fn main() {
    build_rc();
}
