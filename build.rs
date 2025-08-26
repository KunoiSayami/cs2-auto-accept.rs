fn build_rc() {
    #[cfg(not(debug_assertions))]
    embed_resource::compile("build.rc", embed_resource::NONE)
        .manifest_required()
        .unwrap();
}

fn main() {
    build_rc();
}
