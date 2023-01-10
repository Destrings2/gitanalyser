use chrono::NaiveDateTime;
use git2::{Commit, Repository};
use crate::expression_parser::Expr;
use color_eyre::eyre::Result;
use std::collections::HashSet;
use crate::commits::get_modified_files;
use crate::expression_interpreter::evaluate;
use crate::serialization::{CommitData, NativeDateTimeWrapper};

pub struct Analyser {
    pub repo: Repository,
    pub extensions: Vec<String>,
    pub expr: Expr,
    options: AnalyserOptions,
}

#[derive(Debug, Clone)]
pub struct AnalyserOptions {
    pub(crate) evaluate_name: Option<Expr>,
    pub(crate) include_non_tests: bool,
    pub(crate) full_path: bool,
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

        let mut files = Vec::new();
        let mut non_test_files = Vec::new();

        // Get modified files from the commit
        let (modified_files, changed_lines) = get_modified_files(&self.repo, commit, self.options.full_path)?;

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
                    let mut is_test = false;

                    if let Some(evaluate_name) = &self.options.evaluate_name {
                        is_test = is_test || evaluate(evaluate_name, &file.name);
                    }

                    is_test = is_test || evaluate(&self.expr, file_content);

                    if is_test {
                        files.push(file.name);
                    } else if self.options.include_non_tests {
                        non_test_files.push(file.name);
                    }
                }
            }
        }

        if !files.is_empty() {
            Ok(Some(CommitData {
                commit: commit_id.to_string(),
                date: NativeDateTimeWrapper(commit_date_time),
                size: changed_lines,
                test_files: files,
                non_test_files: if self.options.include_non_tests { Some(non_test_files) } else { None },
            }))
        } else {
            Ok(None)
        }
    }
}

// Given a sorted Vec<CommitData>, remove files names, not taking into account the path, that are
// not unique.
pub fn delete_duplicates(commit_data: &[CommitData]) -> Vec<CommitData> {
    let mut seen_files = HashSet::new();
    let mut result = Vec::new();

    for commit in commit_data.iter() {
        let mut files: Vec<Vec<String>> = vec![Vec::new();2];

        let mut handles = vec![&commit.test_files];

        if let Some(non_test_files) = &commit.non_test_files {
            handles.push(non_test_files);
        }

        for (i, handle) in handles.into_iter().enumerate() {
            for file in handle.iter() {
                let path= std::path::Path::new(file.as_str());
                let file_name = path.file_name().unwrap().to_str().unwrap();

                if !seen_files.contains(file_name) {
                    files[i].push(file.clone());
                    seen_files.insert(file_name);
                }
            }
        }

        let mut files = files.into_iter();
        let test_files = files.next().unwrap();
        let non_test_files = files.next().unwrap_or_default();

        if !test_files.is_empty() || !non_test_files.is_empty() {
            result.push(CommitData {
                commit: commit.commit.clone(),
                size: commit.size,
                date: commit.date.clone(),
                test_files,
                non_test_files: if non_test_files.is_empty() { None } else { Some(non_test_files) },
            });
        }
    }

    result
}
