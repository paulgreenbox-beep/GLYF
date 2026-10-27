use glyf_pack::{GlypReader, GlypWriter};
use std::{
    fs::OpenOptions,
    io::ErrorKind,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

struct TempContainer(PathBuf);

impl TempContainer {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        for _ in 0..100 {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "glyf-pack-round-trip-{}-{id}.glyp",
                std::process::id()
            ));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => {
                    drop(file);
                    return Self(path);
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
                Err(error) => panic!("create temporary GLYP path: {error}"),
            }
        }

        panic!("could not reserve a unique temporary GLYP path");
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TempContainer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[test]
fn pack_list_and_extract_preserve_exact_resource_bytes() {
    let manifest = b"glyf { version: 1; }\n";
    let resource = b"abc";
    let path = TempContainer::new();

    let mut writer = GlypWriter::new();
    writer
        .add_file("project.glyf", manifest.to_vec())
        .expect("add manifest");
    writer
        .add_file("assets/sample.bin", resource.to_vec())
        .expect("add resource");
    writer.write(path.path()).expect("write GLYP container");

    let mut reader = GlypReader::open(path.path()).expect("open GLYP container");
    assert_eq!(
        reader.list_files(),
        vec!["project.glyf".to_owned(), "assets/sample.bin".to_owned()]
    );
    assert_eq!(
        reader
            .read_file("project.glyf")
            .expect("extract manifest")
            .as_slice(),
        &manifest[..]
    );
    let extracted = reader
        .read_file("assets/sample.bin")
        .expect("extract resource");
    assert_eq!(extracted.as_slice(), &resource[..]);

}
