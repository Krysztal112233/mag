use super::MinecraftTag;
use crate::tag::Error;
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use std::{ffi::OsStr, path::PathBuf};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MinecraftTagData {
    pub namespace: String,
    pub tag_type: String,
    pub tag_name: String,
    pub tag_data: MinecraftTag,
}

impl MinecraftTagData {
    pub fn try_new(path: &PathBuf) -> super::Result<MinecraftTagData> {
        if !path
            .to_str()
            .map(|it| it.contains("data"))
            .unwrap_or_default()
        {
            trace!("Skipped {}", path.to_str().unwrap());
            return Err(Error::IllegalDirectoryStructure(path.to_path_buf()));
        }

        if !path.extension().map(|it| it == "json").unwrap_or_default() {
            trace!("Skipped {}", path.to_str().unwrap());
            return Err(Error::IllegalDirectoryStructure(path.to_path_buf()));
        }

        let mut base = vec![];

        let mut iter = path.iter().filter_map(OsStr::to_str);
        let (Some(_), Some(namespace), Some(_), Some(tag_type)) = (
            iter.find(|it| *it == "data"),
            iter.next(),
            iter.find(|it| *it == "tags"),
            iter.next(),
        ) else {
            trace!("Illegal directory structure {}", path.to_str().unwrap());
            return Err(Error::IllegalDirectoryStructure(path.to_path_buf()));
        };

        base.extend(iter);

        let tag_name = base.join("/").replace(".json", "");

        debug!(
            "Detected tag \"{}\" with type \"{}\" for namespace \"{}\".",
            tag_name, tag_type, namespace
        );

        let tag = MinecraftTag::try_new(path)?;

        Ok(Self {
            namespace: namespace.into(),
            tag_type: tag_type.into(),
            tag_data: tag,
            tag_name,
        })
    }
}
