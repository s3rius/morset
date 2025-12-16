fn main() {
    let target = std::env::var("TARGET").unwrap();
    if target.contains("windows") {
        let _ = embed_resource::compile("icon.rc", embed_resource::NONE);
    }
}
