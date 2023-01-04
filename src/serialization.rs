// Struct for the following JSON schema:
// {
//     "commit": "string",
//     "date": "string",
//     "files": [string]
// }

use std::collections::HashSet;
use std::io::BufWriter;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use color_eyre::eyre::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct CommitData {
    pub commit: String,
    pub date: NativeDateTimeWrapper,
    pub files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NativeDateTimeWrapper(pub NaiveDateTime);

impl Serialize for NativeDateTimeWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.format("%Y-%m-%d %H:%M:%S UTC").to_string())
    }
}

impl<'de> Deserialize<'de> for NativeDateTimeWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S UTC").unwrap();
        Ok(NativeDateTimeWrapper(dt))
    }
}


pub fn write_to_file(commit_data: &[CommitData], file_path: &str) -> Result<()> {
    let file = std::fs::File::create(file_path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, commit_data)?;
    Ok(())
}

// Given a sorted Vec<CommitData>, remove files names, not taking into account the path, that are
// not unique.
pub fn delete_duplicates(commit_data: &[CommitData]) -> Vec<CommitData> {
    let mut seen_files = HashSet::new();
    let mut result = Vec::new();

    for commit in commit_data.iter() {
        let mut files = Vec::new();
        for file in commit.files.iter() {
            let path= std::path::Path::new(file);
            let file_name = path.file_name().unwrap().to_str().unwrap();

            if !seen_files.contains(file_name) {
                files.push(file.clone());
                seen_files.insert(file_name);
            }
        }

        if !files.is_empty() {
            result.push(CommitData {
                commit: commit.commit.clone(),
                date: commit.date.clone(),
                files,
            });
        }
    }

    result
}