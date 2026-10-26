use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

const MAGIC: &[u8; 4] = b"GLYP";
const FORMAT_VERSION: u32 = 1;

#[derive(Debug)]
pub enum GlypError {
    Io(std::io::Error),
    Invalid(String),
    MissingFile(String),
    DuplicateFile(String),
}

impl std::fmt::Display for GlypError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Invalid(message) => write!(f, "invalid GLYP container: {message}"),
            Self::MissingFile(name) => write!(f, "file '{name}' is not in the GLYP container"),
            Self::DuplicateFile(name) => {
                write!(f, "file '{name}' is already in the GLYP container")
            }
        }
    }
}

impl std::error::Error for GlypError {}

impl From<std::io::Error> for GlypError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Debug)]
struct PendingFile {
    name: String,
    bytes: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct GlypWriter {
    files: Vec<PendingFile>,
}

impl GlypWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(
        &mut self,
        name: impl Into<String>,
        bytes: impl Into<Vec<u8>>,
    ) -> Result<(), GlypError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(GlypError::Invalid("file name cannot be empty".into()));
        }
        if self.files.iter().any(|file| file.name == name) {
            return Err(GlypError::DuplicateFile(name));
        }
        self.files.push(PendingFile {
            name,
            bytes: bytes.into(),
        });
        Ok(())
    }

    pub fn write(&self, path: impl AsRef<Path>) -> Result<(), GlypError> {
        let mut index = Vec::new();
        let header_size = 4 + 4 + 4;
        let index_size: usize = self
            .files
            .iter()
            .map(|file| 4 + file.name.len() + 8 + 8)
            .sum();
        let mut offset = (header_size + index_size) as u64;
        for file in &self.files {
            let name_bytes = file.name.as_bytes();
            write_u32(&mut index, name_bytes.len() as u32)?;
            index.extend_from_slice(name_bytes);
            write_u64(&mut index, offset)?;
            write_u64(&mut index, file.bytes.len() as u64)?;
            offset += file.bytes.len() as u64;
        }

        let mut output = File::create(path)?;
        output.write_all(MAGIC)?;
        write_u32(&mut output, FORMAT_VERSION)?;
        write_u32(&mut output, self.files.len() as u32)?;
        output.write_all(&index)?;
        for file in &self.files {
            output.write_all(&file.bytes)?;
        }
        output.flush()?;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct IndexEntry {
    name: String,
    offset: u64,
    size: u64,
}

#[derive(Debug)]
pub struct GlypReader {
    path: PathBuf,
    file: File,
    entries: Vec<IndexEntry>,
}

impl GlypReader {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, GlypError> {
        let path = path.as_ref().to_path_buf();
        let mut file = File::open(&path)?;
        let mut magic = [0_u8; 4];
        file.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(GlypError::Invalid("missing GLYP magic header".into()));
        }
        let version = read_u32(&mut file)?;
        if version != FORMAT_VERSION {
            return Err(GlypError::Invalid(format!("unsupported version {version}")));
        }
        let count = read_u32(&mut file)? as usize;
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            let name_length = read_u32(&mut file)? as usize;
            if name_length > 16 * 1024 * 1024 {
                return Err(GlypError::Invalid("file name is too large".into()));
            }
            let mut name_bytes = vec![0_u8; name_length];
            file.read_exact(&mut name_bytes)?;
            let name = String::from_utf8(name_bytes)
                .map_err(|_| GlypError::Invalid("file name is not UTF-8".into()))?;
            let offset = read_u64(&mut file)?;
            let size = read_u64(&mut file)?;
            entries.push(IndexEntry { name, offset, size });
        }
        let file_length = file.metadata()?.len();
        for entry in &entries {
            let end = entry
                .offset
                .checked_add(entry.size)
                .ok_or_else(|| GlypError::Invalid("file range overflows".into()))?;
            if end > file_length {
                return Err(GlypError::Invalid(format!(
                    "file '{}' points outside the container",
                    entry.name
                )));
            }
        }
        Ok(Self {
            path,
            file,
            entries,
        })
    }

    pub fn list_files(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|entry| entry.name.clone())
            .collect()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.entries.iter().any(|entry| entry.name == name)
    }

    pub fn read_file(&mut self, name: &str) -> Result<Vec<u8>, GlypError> {
        let entry = self
            .entries
            .iter()
            .find(|entry| entry.name == name)
            .cloned()
            .ok_or_else(|| GlypError::MissingFile(name.into()))?;
        self.file.seek(SeekFrom::Start(entry.offset))?;
        let mut bytes = vec![0_u8; entry.size as usize];
        self.file.read_exact(&mut bytes)?;
        Ok(bytes)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

fn write_u32(writer: &mut impl Write, value: u32) -> Result<(), GlypError> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn write_u64(writer: &mut impl Write, value: u64) -> Result<(), GlypError> {
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn read_u32(reader: &mut impl Read) -> Result<u32, GlypError> {
    let mut bytes = [0_u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u64(reader: &mut impl Read) -> Result<u64, GlypError> {
    let mut bytes = [0_u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writer_and_reader_keep_files_directly_readable() {
        let path = std::env::temp_dir().join(format!("glyf-pack-{}.glyp", std::process::id()));
        let mut writer = GlypWriter::new();
        writer
            .add_file("main_menu.jpg", vec![1, 2, 3, 4])
            .expect("add image");
        writer.add_file("icon.ico", vec![9, 8]).expect("add icon");
        writer.write(&path).expect("write container");

        let mut reader = GlypReader::open(&path).expect("open container");
        assert_eq!(reader.list_files(), vec!["main_menu.jpg", "icon.ico"]);
        assert_eq!(
            reader.read_file("main_menu.jpg").expect("read image"),
            vec![1, 2, 3, 4]
        );
        assert_eq!(reader.read_file("icon.ico").expect("read icon"), vec![9, 8]);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn duplicate_names_are_rejected() {
        let mut writer = GlypWriter::new();
        writer.add_file("same", vec![1]).expect("first file");
        assert!(matches!(
            writer.add_file("same", vec![2]),
            Err(GlypError::DuplicateFile(_))
        ));
    }
}
