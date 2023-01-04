use chrono::NaiveDateTime;
use git2::{Commit, Repository};
use crate::expression_parser::Expr;
use color_eyre::eyre::Result;
use crate::commits::get_modified_files;
use crate::expression_interpreter::evaluate;
use crate::serialization::{CommitData};

pub struct Analyser {
    pub repo: Repository,
    pub extensions: Vec<String>,
    pub expr: Expr,
    options: AnalyserOptions,
}

#[derive(Debug, Clone)]
pub struct AnalyserOptions {
    pub(crate) evaluate_name: bool,
    pub(crate) evaluate_content: bool,
}

impl Analyser {
    pub fn new(repo: Repository, extensions: Vec<String>, expr: Expr, opts: AnalyserOptions) -> Analyser {
        Analyser {
            repo,
            extensions,
            expr,
            options: opts,
        }
    }

    pub fn process_commit(&self, commit: &Commit) -> Result<Option<CommitData>> {
        let commit_id = commit.id();
        let commit_date_time = NaiveDateTime::from_timestamp_opt(commit.time().seconds(), 0).unwrap();
        let commit_utc = commit_date_time.format("%Y-%m-%d %H:%M:%S UTC").to_string();

        let mut files = Vec::new();

        // Get modified files from the commit
        let modified_files = get_modified_files(&self.repo, commit)?;

        for file in modified_files {
            // Only consider new files
            if file.modification_type != git2::Delta::Added {
                continue;
            }

            // Get the referenced blob object
            let blob = self.repo.find_blob(file.oid);
            if blob.is_err() {
                continue;
            }

            let blob = blob.unwrap();

            // Get the file extension
            let extension = std::path::Path::new(&file.name).extension();

            // If it has an extension check it against the list of extensions
            if let Some(extension) = extension {
                let extension = extension.to_os_string().into_string().unwrap();
                if self.extensions.contains(&extension) {
                    // Read the blob content as utf8
                    let file_content = std::str::from_utf8(blob.content());
                    if file_content.is_err() {
                        continue;
                    }

                    let file_content = file_content.unwrap();

                    // Include the file if it matches the expression
                    let mut include = false;

                    if self.options.evaluate_name {
                        include = include || evaluate(&self.expr, &file.name);
                    }

                    if self.options.evaluate_content {
                        include = include || evaluate(&self.expr, file_content);
                    }

                    if include {
                        files.push(file.name.clone());
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