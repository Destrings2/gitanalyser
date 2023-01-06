use chrono::{NaiveDate, NaiveDateTime};
use git2::{Commit, Repository, Sort};
use color_eyre::eyre::Result;

/// Returns an iterator over the commits in the repository for a given branch.
/// The commits are returned in chronological order.
/// An optional start and end date can be specified.
/// An optional start and end commit can be specified.
pub fn get_commit_walker<'a>(
    repo: &'a Repository,
    branch: &str,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    start_commit: Option<String>,
) -> Result<impl Iterator<Item=Commit<'a>>, git2::Error> {

    let possible_branch_names = branch.split('|').collect::<Vec<&str>>();

    // Initialize the walker
    let mut revwalk = repo.revwalk()?;


    // Set sorting to chronological order
    revwalk.set_sorting(Sort::TIME)?;

    // If a start commit is specified, use it
    if let Some(start_commit) = start_commit {
        revwalk.push(git2::Oid::from_str(&start_commit)?)?;
    }

    // Use the branch reference as the starting point
    let mut has_branch = false;
    for branch_name in possible_branch_names {
        let branch_ref = repo.find_branch(branch_name, git2::BranchType::Local);
        if let Ok(branch) = branch_ref {
            let branch_commit = branch.get().target().unwrap();
            revwalk.push(branch_commit)?;
            has_branch = true;
            break;
        }
    }

    if !has_branch {
        return Err(git2::Error::from_str("No branch found"));
    }

    // Transform the oids into commits and filter out commits that are not in the specified date range
    let commits = revwalk
        .filter_map(|oid| oid.ok())
        .map(|oid| repo.find_commit(oid))
        .filter_map(|commit| commit.ok())
        .filter(move |commit| {
            let commit_date_seconds = commit.time().seconds();
            let commit_date = NaiveDateTime::from_timestamp_opt(commit_date_seconds, 0)
                .map(|dt| dt.date()).unwrap();

            if let Some(start_date) = start_date {
                if commit_date < start_date {
                    return false;
                }
            }

            if let Some(end_date) = end_date {
                if commit_date > end_date {
                    return false;
                }
            }

            true
        });

    Ok(commits)
}

#[derive(Debug)]
pub struct CommitFile {
    pub name: String,
    pub oid: git2::Oid,
    pub modification_type: git2::Delta
}

pub fn get_modified_files(repo: &Repository, commit: &Commit, full_names: bool) -> Result<(Vec<CommitFile>, usize)> {
    let commit_tree = commit.tree()?;
    let n_parents = commit.parent_count();

    if n_parents == 1 {
        let parent_commit = commit.parent(0)?;
        let parent_tree = parent_commit.tree()?;
        let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None)?;
        let diff_deltas = diff.deltas();

        let stats = diff.stats()?;
        let changed_lines = stats.insertions() + stats.deletions();

        let mut modified_files = Vec::new();

        for delta in diff_deltas {

            let file_name = if full_names {
                delta.new_file().path().unwrap().to_str().unwrap().to_string()
            } else {
                delta.new_file().path().unwrap().file_name().unwrap().to_str().unwrap().to_string()
            };

            modified_files.push(CommitFile {
                name: file_name,
                oid: delta.new_file().id(),
                modification_type: delta.status(),
            });
        }

        Ok((modified_files, changed_lines))
    } else {
        // This is a merge commit
        Ok((Vec::new(), 0))
    }
}

// Wraps the commit data with Send + Sync
#[derive(Debug, Clone)]
pub struct CommitSendSync<'a> {
    pub commit: Commit<'a>
}

unsafe impl<'a> Send for CommitSendSync<'a> {}
unsafe impl<'a> Sync for CommitSendSync<'a> {}
