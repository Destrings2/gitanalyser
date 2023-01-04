use chrono::NaiveDateTime;
use git2::{Commit, Repository};
use crate::expression_parser::Expr;
use color_eyre::eyre::Result;
use crate::commits::get_modified_files;
use crate::expression_interpreter::evaluate;
use crate::serialization::{CommitData, File};

pub struct Analyser<'a> {
    pub repo: &'a Repository,
    pub extensions: Vec<String>,
    pub expr: Expr,
}

impl<'a> Analyser<'a> {
    pub fn new(repo: &'a Repository, extensions: Vec<String>, expr: Expr) -> Analyser {
        Analyser {
            repo,
            extensions,
            expr,
        }
    }

    pub fn process_commit(&self, commit: &Commit) -> Result<Option<CommitData>> {
        let commit_id = commit.id();
        let commit_date_time = NaiveDateTime::from_timestamp_opt(commit.time().seconds(), 0).unwrap();
        let commit_utc = commit_date_time.format("%Y-%m-%d %H:%M:%S UTC").to_string();

        let mut files = Vec::new();

        let modified_files = get_modified_files(self.repo, commit)?;

        for file in modified_files {
            if file.modification_type != git2::Delta::Added {
                continue;
            }

            let blob = self.repo.find_blob(file.oid)?;

            let extension = std::path::Path::new(&file.name).extension();

            if let Some(extension) = extension {
                let extension = extension.to_os_string().into_string().unwrap();
                if self.extensions.contains(&extension) {
                    let file_content = std::str::from_utf8(blob.content()).unwrap();

                    if evaluate(&self.expr, file_content) {
                        files.push(File {
                            name: file.name.clone(),
                            oid: file.oid.to_string(),
                        });
                    }
                }
            }
        }

        if !files.is_empty() {
            Ok(Some(CommitData {
                commit: commit_id.to_string(),
                date: commit_utc,
                files,
            }))
        } else {
            Ok(None)
        }
    }
}