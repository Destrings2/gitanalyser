use clap::Parser;
use chrono::prelude::*;
use color_eyre::eyre::Result;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
pub struct Arguments {
    /// Path to the git repository
    #[clap(short, long, default_value=".")]
    pub path: String,

    /// The branch to analyse
    #[clap(short, long, default_value="master")]
    pub branch: String,

    /// Whether to append to the output file
    #[clap(short, long, default_value="false")]
    pub append: bool,

    /// Date to start the analysis from (format: YYYY-MM-DD)
    ///
    /// If not specified, the analysis will start from the beginning of the repository
    #[clap(long, value_parser=parse_date)]
    pub start_date: Option<NaiveDate>,

    /// Commit to start the analysis from, given as a hash.
    /// The commits will be analysed in chronological order, starting from the commit with the
    /// given hash.
    ///
    /// If not specified, the analysis will start from the beginning of the repository
    ///
    /// If both start_date and start_commit are specified, the analysis will start from the
    /// commit with the given hash, if it is newer than the given date.
    #[clap(long)]
    pub start_commit: Option<String>,

    /// Date to end the analysis at (format: YYYY-MM-DD)
    ///
    /// If not specified, the analysis will end at the end of the repository
    #[clap(long, value_parser=parse_date)]
    pub end_date: Option<NaiveDate>,

    /// File extensions to include in the analysis
    #[clap(short, long, default_value="java")]
    pub extensions: Vec<String>,

    /// Number of threads to use for the analysis
    /// defaults to the number of logical cores
    #[clap(short, long)]
    pub threads: Option<usize>,

    /// Compare regex against the file name
    #[clap(long)]
    pub evaluate_name: Option<String>,

    /// Include non-test files in the analysis
    #[clap(long, default_value="false")]
    pub save_non_tests: bool,

    /// Delete duplicate files from the analysis
    #[clap(long, default_value="false")]
    pub delete_duplicates: bool,

    /// Boolean combination of regular expressions to match the files to analyse.
    /// A regular expression is of the form /regex/.
    /// The files are included in the results if the expression evaluates to true.
    /// AND, OR and NOT are supported.
    ///
    /// AND(expression1, expression2, ...) - all expressions must evaluate to true
    ///
    /// OR(expression1, expression2, ...) - at least one expression must evaluate to true
    ///
    /// NOT(expression) - the expression must evaluate to false
    ///
    /// AND(OR(expression1, expression2), NOT(expression3)) - a combination of AND, OR and NOT
    pub regex_pattern: String,

    /// Output file
    pub output: String,
}

fn parse_date(s: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").
        map_err(|e| color_eyre::eyre::eyre!("Invalid date format: {}", e))
}