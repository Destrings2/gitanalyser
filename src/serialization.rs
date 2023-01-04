// Struct for the following JSON schema:
// {
//     "commit": "string",
//     "date": "string",
//     "files": [string]
// }

use std::io::BufWriter;
use serde::{Deserialize, Serialize};
use color_eyre::eyre::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct CommitData {
    pub commit: String,
    pub date: String,
    pub files: Vec<String>,
}

pub fn write_to_file(commit_data: &[CommitData], file_path: &str) -> Result<()> {
    let file = std::fs::File::create(file_path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, commit_data)?;
    Ok(())
}