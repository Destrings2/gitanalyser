// Struct for the following JSON schema:
// {
//     "commit": "string",
//     "date": "string",
//     "files": [string]
// }

use std::io::{BufWriter};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use color_eyre::eyre::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct CommitData {
    pub commit: String,
    pub date: NativeDateTimeWrapper,
    pub size: usize,
    pub test_files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_test_files: Option<Vec<String>>,
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