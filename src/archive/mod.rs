use log::debug;
use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};
use temp_dir::TempDir;
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};
use zip::{result::ZipError, ZipArchive};

pub mod fabric;
pub mod forge;

#[derive(Debug, Error)]
pub enum ModArchiveError {
    #[error("{0}")]
    Zip(#[from] ZipError),

    #[error("{0}")]
    NotFile(PathBuf),

    #[error("{0}")]
    IO(#[from] io::Error),
}

type Result<T> = std::result::Result<T, ModArchiveError>;

fn decompress<T>(archive: T) -> Result<TempDir>
where
    T: AsRef<Path>,
{
    let temp_dir = TempDir::new().unwrap();

    debug!(
        "Decompress jar {} to temporary directory {}.",
        archive.as_ref().to_str().unwrap(),
        temp_dir.path().to_str().unwrap()
    );

    if archive.as_ref().is_dir() {
        return Err(ModArchiveError::NotFile(archive.as_ref().to_path_buf()));
    }

    let archive = File::open(archive)?;
    let mut zip = ZipArchive::new(archive)?;
    zip.extract(temp_dir.path())?;

    Ok(temp_dir)
}

fn flatten(temp_dir: &TempDir) -> Vec<TempDir> {
    let jars: Vec<_> = WalkDir::new(temp_dir.path())
        .max_depth(4)
        .into_iter()
        .flatten()
        .filter(|it| it.path().is_file())
        .filter(|it| it.path().extension().is_some_and(|ext| ext == "jar"))
        .collect();

    for ele in jars.iter() {
        debug!(
            "Detected jar file {}.",
            ele.to_owned().into_path().to_str().unwrap()
        );
    }

    let decompressed: Vec<_> = jars
        .iter()
        .map(DirEntry::path)
        .flat_map(decompress)
        .collect();

    decompressed
}

pub fn flatten_decompress<T>(file: T) -> Result<Vec<TempDir>>
where
    T: AsRef<Path>,
{
    let file = decompress(file)?;
    let mut result = flatten(&file);

    result.push(file);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_decompress() {
        let file = PathBuf::from_iter([
            env!("CARGO_MANIFEST_DIR"),
            "test",
            "fixture",
            "jar",
            "api.jar",
        ]);

        assert_eq!(flatten_decompress(file).unwrap().len(), 49)
    }
}
