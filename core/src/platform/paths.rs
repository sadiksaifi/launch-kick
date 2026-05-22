use std::path::PathBuf;

pub fn launcher_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("core should live inside the repo root")
        .join(".build/launch-kick")
}
