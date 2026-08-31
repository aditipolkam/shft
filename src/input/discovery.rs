use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::{
    codecs::ImageFormat,
    error::{Result, ShftError},
};

pub fn discover(input: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    if input.is_file() {
        return if ImageFormat::from_path(input).is_some() {
            Ok(vec![input.to_path_buf()])
        } else {
            Err(ShftError::UnsupportedFormat(input.to_path_buf()))
        };
    }
    if !input.is_dir() {
        return Err(ShftError::Discovery {
            path: input.to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "path does not exist"),
        });
    }
    let depth = if recursive { usize::MAX } else { 1 };
    let mut files: Vec<_> = WalkDir::new(input)
        .min_depth(1)
        .max_depth(depth)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            entry.file_type().is_file() && ImageFormat::from_path(entry.path()).is_some()
        })
        .map(|entry| entry.into_path())
        .collect();
    files.sort();
    if files.is_empty() {
        Err(ShftError::NoImages(input.to_path_buf()))
    } else {
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ignores_unsupported_and_respects_recursion() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("a.png"), b"").unwrap();
        std::fs::write(root.path().join("notes.txt"), b"").unwrap();
        std::fs::create_dir(root.path().join("nested")).unwrap();
        std::fs::write(root.path().join("nested/b.jpg"), b"").unwrap();
        assert_eq!(discover(root.path(), false).unwrap().len(), 1);
        assert_eq!(discover(root.path(), true).unwrap().len(), 2);
    }
}
