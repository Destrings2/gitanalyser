use std::collections::HashMap;
use std::env;
use std::io::BufWriter;
use gitanalyser::serialization::{CommitData, read_from_file};
use serde::{Serialize, Deserialize};

#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestData {
    pub test_file: String,
    pub test_file_date: String,
    pub test_file_size: usize,
    pub tested_file: String,
    pub tested_file_date: String,
    pub tested_file_size: usize,
}

pub fn write_to_file(test_data: &[TestData], file_path: &str) {
    let file = std::fs::File::create(file_path).unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, test_data).unwrap();
}

fn main() {
    // Read one argument from the command line
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: testlinker <file> <output>");
        return;
    }

    // Open the file
    let commits: Vec<CommitData> = read_from_file(&args[1]).expect("Could not read file");
    println!("Commits length: {}", commits.len());

    // Each CommitData contains a list of test files and a list of non-test files

    // Gather all test files
    let mut test_files: HashMap<String, Vec<String>> = HashMap::new();
    for commit in &commits {
        for file in &commit.test_files {
            let name = file.clone();
            let date_string = commit.date.0.format("%Y-%m-%d %H:%M:%S+00:00").to_string();
            test_files.insert(name, vec![commit.size.to_string(), date_string]);
        }
    }

    // Gather all non-test files
    let mut non_test_files: HashMap<String, Vec<String>> = HashMap::new();
    for commit in &commits {
        if let Some(files) = &commit.non_test_files {
            for file in files {
                let name = file.clone();
                let date_string = commit.date.0.format("%Y-%m-%d %H:%M:%S+00:00").to_string();
                non_test_files.insert(name, vec![commit.size.to_string(), date_string]);
            }
        }
    }

    let mut output = Vec::new();
    for (non_test_file, attributes) in non_test_files.into_iter() {
        let test_after = format!("{}{}", non_test_file, "Test");
        let test_before = format!("{}{}", "Test", non_test_file);

        let test_file = if test_files.contains_key(&test_before) {
            test_before
        } else if test_files.contains_key(&test_after) {
            test_after
        } else {
            continue;
        };

        let data = TestData {
            test_file: test_file.clone(),
            test_file_date: test_files.get(&test_file).unwrap()[1].clone(),
            test_file_size: test_files.get(&test_file).unwrap()[0].parse().unwrap(),
            tested_file: non_test_file.clone(),
            tested_file_date: attributes[1].clone(),
            tested_file_size: attributes[0].parse().unwrap(),
        };

        output.push(data);
    }

    write_to_file(&output, &args[2]);
}
