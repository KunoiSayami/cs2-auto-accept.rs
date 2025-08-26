fn build_rc() {
    #[cfg(not(debug_assertions))]
    match embed_resource::compile("build.rc", embed_resource::NONE) {
        embed_resource::CompilationResult::NotWindows | embed_resource::CompilationResult::Ok => {}
        embed_resource::CompilationResult::NotAttempted(cow)
        | embed_resource::CompilationResult::Failed(cow) => {
            eprintln!("Build rc error: {cow}")
        }
    }
}

fn main() {
    build_rc();
}
