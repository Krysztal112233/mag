use data::MinecraftTagData;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufReader},
    path::{Path, PathBuf},
};
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

pub mod data;

#[derive(Debug, Error)]
pub enum Error {
    #[error("illegal directory structure: {0}")]
    IllegalDirectoryStructure(PathBuf),

    #[error("{0}")]
    IO(#[from] io::Error),

    #[error("{0}")]
    Serde(#[from] serde_json::Error),
}

pub(crate) type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MinecraftTag {
    #[serde(default = "bool::default")]
    replace: bool,
    values: Vec<MinecraftTagValue>,
}

impl MinecraftTag {
    pub fn try_new(path: &PathBuf) -> Result<MinecraftTag> {
        Ok(serde_json::from_reader(BufReader::new(File::open(path)?))?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(untagged)]
pub enum MinecraftTagValue {
    Plain(String),
    Complex { id: String, required: bool },
}

impl MinecraftTagValue {
    pub fn is_plain(&self) -> bool {
        matches!(self, Self::Plain(_))
    }

    pub fn is_tag(&self) -> bool {
        match self {
            Self::Plain(s) => s,
            Self::Complex { id, .. } => id,
        }
        .starts_with('#')
    }

    pub fn to_plain(&self) -> Self {
        let id = match self {
            MinecraftTagValue::Plain(s) => s,
            MinecraftTagValue::Complex { id, required: _ } => id,
        };

        Self::Plain(id.into())
    }
}

impl MinecraftTag {
    pub fn flatten_value(&self) -> Vec<String> {
        self.values
            .iter()
            .map(|it| match it {
                MinecraftTagValue::Complex { id, required: _ } => id,
                MinecraftTagValue::Plain(s) => s,
            })
            .cloned()
            .collect()
    }
}

pub async fn walk<T>(root: T) -> impl Iterator<Item = MinecraftTagData>
where
    T: AsRef<Path>,
{
    let entry: Vec<_> = WalkDir::new(root)
        .into_iter()
        .par_bridge()
        .flatten()
        .filter(|it| it.path().is_file())
        .map(DirEntry::into_path)
        .collect();

    entry
        .into_iter()
        .flat_map(|path: PathBuf| MinecraftTagData::try_new(&path))
}

pub fn merge(tags: Vec<Vec<MinecraftTagData>>) -> Vec<MinecraftTagData> {
    let tags = tags.into_iter().flatten().collect::<Vec<_>>();

    let mut merged: Vec<MinecraftTagData> = Vec::new();

    for ele in &tags {
        let f = |it: &MinecraftTagData| {
            *it.namespace == ele.namespace
                && *it.tag_name == ele.tag_name
                && *it.tag_type == it.tag_type
        };

        if merged.iter().any(f) {
            continue;
        }

        let r = tags.iter().filter(|it| f(it)).fold(
            MinecraftTagData {
                namespace: ele.namespace.to_string(),
                tag_type: ele.tag_type.to_string(),
                tag_name: ele.tag_name.to_string(),
                ..Default::default()
            },
            |mut acc, it| {
                let mut it_data_value = it
                    .tag_data
                    .values
                    .iter()
                    .map(MinecraftTagValue::to_plain)
                    .chain(acc.tag_data.values)
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();

                it_data_value.sort();

                acc.tag_data.values = it_data_value;
                acc
            },
        );

        merged.push(r)
    }

    merged
}
