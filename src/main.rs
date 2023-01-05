extern crate core;

mod arguments;
mod expression_parser;
mod expression_interpreter;
mod commits;
mod analysis;

use clap::Parser;
use color_eyre::eyre::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use crate::commits::CommitSendSync;
use rayon::prelude::*;
use analysis::delete_duplicates;
use gitanalyser::serialization::write_to_file;

fn main() -> Result<()>{
    color_eyre::install()?;

    // Parse arguments
    let args = arguments::Arguments::parse();

    // Open repository
    let repository = git2::Repository::open(&args.path)?;

    // Initialize the git repository walker
    let walker = commits::get_commit_walker(
        &repository,
        &args.branch,
        args.start_date,
        args.end_date,
        args.start_commit,
    )?;

    // Parse the regular expressions tree
    let expr = expression_parser::parse(args.regex_pattern.as_str())?;

    println!("Starting analysis...");
    println!("Considering files with extensions: {:?}", args.extensions);

    // Wrap the commits in a SendSync wrapper so it can be used in parallel
    let commits: Vec<CommitSendSync> = walker.map(|commit| CommitSendSync {
        commit
    }).collect();

    // Obtain number of available cores
    let num_cores = num_cpus::get();
    let num_processes = args.threads.unwrap_or(num_cores).min(num_cores);

    // Initialize progress bar
    let m = MultiProgress::new();
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}",
    ).unwrap().progress_chars("##-");

    // Initialize Rayon with the number of cores
    rayon::ThreadPoolBuilder::new().num_threads(num_processes).build_global().unwrap();

    // Split the commits into chunks
    let chunk_size = (commits.len() as f32 / num_processes as f32).ceil() as usize;
    let chunks: Vec<Vec<CommitSendSync>> = commits.chunks(chunk_size).map(|chunk| chunk.to_vec()).collect();

    // Create a progress bar for each chunk
    let mut chunks_and_progress= Vec::new();
    for chunk in chunks.into_iter() {
        let progress_bar = m.add(ProgressBar::new(chunk.len() as u64));
        progress_bar.set_style(sty.clone());
        chunks_and_progress.push((chunk, progress_bar));
    }

    // Mutex output vector, so it can be used in parallel
    let files = std::sync::Mutex::new(Vec::new());

    // Initialize the analyser options
    let evaluate_name_expr = args.evaluate_name.map(|expr| expression_parser::parse(expr.as_str()).unwrap());
    let analyser_opts = analysis::AnalyserOptions {
        evaluate_name: evaluate_name_expr,
        include_non_tests: args.save_non_tests,
    };

    // Analyse each chunk in parallel
    chunks_and_progress.par_iter().for_each(|(chunk, pb)| {
        // Open a repository and clone the other arguments to create an analyser
        let repo = git2::Repository::open(&args.path).unwrap();
        let analyser = analysis::Analyser::new(repo, args.extensions.clone(), expr.clone(), analyser_opts.clone());

        // Store the results in a temporary vector
        let mut commit_data = Vec::new();

        // Analyse each commit in the chunk
        for commit in chunk {
            let commit_datum = analyser.process_commit(&commit.commit).unwrap();

            // If the commit has relevant data, add it to the temporary vector
            if let Some(commit_datum) = commit_datum {
                commit_data.push(commit_datum);
            }

            // Increment the progress bar
            pb.inc(1);
        }

        // Lock and append the temporary vector to the output vector
        files.lock().unwrap().extend(commit_data);
    });

    m.clear()?;

    // Sort the commits by date
    println!("Sorting commits...");
    let mut files = files.into_inner().unwrap();
    files.sort_by(|a, b| a.date.0.cmp(&b.date.0));

    // Delete duplicates
    let files_to_write = if args.delete_duplicates {
        println!("Deleting duplicates...");
        delete_duplicates(&files)
    } else {
        files
    };

    // Write the output to a file
    println!("Writing output to file...");
    write_to_file(&files_to_write, args.output.as_str())?;
    println!("Done!");

    Ok(())
}
