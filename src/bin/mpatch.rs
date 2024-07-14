use std::{env, path::PathBuf};

use clap::Parser;
use mpatch::{filtering::DistanceFilter, patch::PatchPaths, LCSMatcher};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let matcher = LCSMatcher;
    let filter = DistanceFilter::new(cli.max_match_distance);

    let patch_paths = PatchPaths::new(
        cli.source_dir.into(),
        env::current_dir()?,
        PathBuf::from(cli.patch_file),
        cli.rejects_file.map(PathBuf::from),
    );

    if let Err(error) = mpatch::apply_all(patch_paths, cli.strip, cli.dryrun, matcher, filter) {
        eprintln!("{}", error);
        return Err(Box::new(error));
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
    #[arg(long = "max_match_distance", default_value_t = 0)]
    max_match_distance: usize,
    #[arg(long = "dryrun", default_value_t = false)]
    dryrun: bool,
}
