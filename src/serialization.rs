// Struct for the following JSON schema:
// {
//     "commit": "string",
//     "date": "string",
//     "files": [string]
// }

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub(crate) oid: String,
    pub(crate) name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommitData {
    pub commit: String,
    pub date: String,
    pub files: Vec<File>,
}