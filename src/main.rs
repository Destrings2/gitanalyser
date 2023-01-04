extern crate core;

mod arguments;
mod expression_parser;
mod expression_interpreter;
mod commits;
mod serialization;
mod analysis;

use chrono::NaiveDateTime;
use clap::Parser;
use color_eyre::eyre::Result;
use crate::expression_interpreter::evaluate;
use crate::serialization::{CommitData, File};

fn main() -> Result<()>{
    color_eyre::install()?;
    let args = arguments::Arguments::parse();

    let repository = git2::Repository::open(&args.path)?;

    let walker = commits::get_commit_walker(
        &repository,
        &args.branch,
        args.start_date,
        args.end_date,
        args.start_commit,
    )?;

    let expr = expression_parser::parse(args.regex_pattern.as_str())?;

    println!("Starting analysis...");
    println!("Considering files with extensions: {:?}", args.extensions);

    let mut commits: Vec<CommitData> = Vec::new();
    // Send each commit to a thread to be analysed using crossbeam
    
    

    for commit in walker {
        let commit_id = commit.id();
        let commit_date_time = NaiveDateTime::from_timestamp_opt(commit.time().seconds(), 0).unwrap();
        let commit_utc = commit_date_time.format("%Y-%m-%d %H:%M:%S UTC").to_string();

        let mut files = Vec::new();

        let modified_files = commits::get_modified_files(&repository, &commit)?;

        for file in modified_files {
            if file.modification_type != git2::Delta::Added {
                continue;
            }

            let blob = repository.find_blob(file.oid)?;

            let extension = std::path::Path::new(&file.name).extension();

            if let Some(extension) = extension {
                let extension = extension.to_os_string().into_string().unwrap();
                if args.extensions.contains(&extension) {
                    let file_content = std::str::from_utf8(blob.content()).unwrap();

                    if evaluate(&expr, file_content) {
                        files.push(File {
                            name: file.name.clone(),
                            oid: file.oid.to_string(),
                        });
                    }
                }
            }
        }

        if !files.is_empty() {
            println!("Found commit {} at {} with {} test files", commit_id, commit_utc, files.len());
            commits.push(CommitData {
                commit: commit_id.to_string(),
                date: commit_utc.clone(),
                files,
            });
        }
    }

    Ok(())
}
