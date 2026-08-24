use std::{fs, io::Write, path::Path};

use crate::error::{Result, ShftError};

pub fn atomic_write(path: &Path, bytes: &[u8], overwrite: bool) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| ShftError::Write {
        path: path.to_path_buf(),
        source,
    })?;
    if path.exists() && !overwrite {
        return Err(ShftError::OutputExists(path.to_path_buf()));
    }
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).map_err(|source| ShftError::Write {
            path: path.to_path_buf(),
            source,
        })?;
    temporary
        .write_all(bytes)
        .and_then(|_| temporary.as_file().sync_all())
        .map_err(|source| ShftError::Write {
            path: path.to_path_buf(),
            source,
        })?;
    let result = if overwrite {
        temporary.persist(path)
    } else {
        temporary.persist_noclobber(path)
    };
    result.map_err(|error| ShftError::Write {
        path: path.to_path_buf(),
        source: error.error,
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_overwrite_and_leaves_original() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x");
        fs::write(&path, b"old").unwrap();
        assert!(atomic_write(&path, b"new", false).is_err());
        assert_eq!(fs::read(path).unwrap(), b"old");
    }
    #[test]
    fn atomically_writes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x");
        atomic_write(&path, b"complete", false).unwrap();
        assert_eq!(fs::read(path).unwrap(), b"complete");
    }
}
