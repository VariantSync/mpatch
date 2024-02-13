use std::{env, error::Error, path::PathBuf, str::FromStr};

// TODO: write rejects to file

use clap::Parser;
use mpatch::{
    files::StrippedPath, patch::FilePatch, CommitDiff, FileArtifact, LCSMatcher, Matcher,
};

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let mut matcher = LCSMatcher;

    let diff = CommitDiff::read(&cli.patch_file).unwrap();
    let source_dir = PathBuf::from_str(&cli.source_dir)
        .expect("was not able to parse path to source directory (<SOURCEDIR>)");
    let target_dir = env::current_dir()?;

    for file_diff in diff {
        let mut source_file_path = source_dir.clone();
        source_file_path.push(PathBuf::from_stripped(
            &file_diff.source_file().path(),
            cli.strip,
        ));

        let mut target_file_path = target_dir.clone();
        target_file_path.push(PathBuf::from_stripped(
            &file_diff.target_file().path(),
            cli.strip,
        ));

        let source = FileArtifact::read(
            source_file_path
                .to_str()
                .expect("the source directory is not a valid UTF-8 path"),
        )
        .unwrap();
        let target = FileArtifact::read(
            target_file_path
                .to_str()
                .expect("the target directory is not a valid UTF-8 path"),
        )
        .unwrap();

        let matching = matcher.match_files(source, target);
        let patch = FilePatch::from(file_diff);
        let aligned_patch = patch.align_to_target(matching);
        let actual_result = aligned_patch.apply();
        let (actual_result, rejects) = (
            actual_result.patched_file(),
            actual_result.rejected_changes(),
        );

        if !cli.dryrun {
            actual_result.write()?;
        }

        if !rejects.is_empty() {
            eprintln!("{rejects:?}");
        }
    }

    Ok(())
}

#[derive(Parser)]
struct Cli {
    #[arg(long = "sourcedir")]
    source_dir: String,
    #[arg(long = "patchfile")]
    patch_file: String,
    #[arg(long = "rejectsfile")]
    rejects_file: Option<String>,
    #[arg(long = "strip", default_value_t = 0)]
    strip: usize,
    #[arg(long = "dryrun", default_value_t = false)]
    dryrun: bool,
}
