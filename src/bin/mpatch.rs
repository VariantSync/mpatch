use std::{env, path::PathBuf};

use clap::Parser;
use mpatch::{
    filtering::{DistanceFilter, InsideMatchFilter},
    patch::PatchPaths,
    LCSMatcher,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let matcher = LCSMatcher;

    let patch_paths = PatchPaths::new(
        cli.source_dir.into(),
        env::current_dir()?,
        PathBuf::from(cli.patch_file),
        cli.rejects_file.map(PathBuf::from),
    );

    let result = match cli.filter.as_str() {
        "distance" => {
            let filter = DistanceFilter::new(cli.match_distance_cutoff);
            mpatch::apply_all(patch_paths, cli.strip, cli.dryrun, matcher, filter)
        }
        "match" => {
            let filter = InsideMatchFilter::new(cli.match_distance_cutoff);
            mpatch::apply_all(patch_paths, cli.strip, cli.dryrun, matcher, filter)
        }
        _ => {
            panic!("Invalid filter type");
        }
    };

    if let Err(error) = result {
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
    #[arg(long = "match_distance_cutoff", default_value_t = 2)]
    match_distance_cutoff: usize,
    #[arg(long = "dryrun", default_value_t = false)]
    dryrun: bool,
    #[arg(long = "filter", default_value = "distance")]
    filter: String,
}
