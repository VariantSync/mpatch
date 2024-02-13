use std::env;

use clap::Parser;
use mpatch::LCSMatcher;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let matcher = LCSMatcher;

    mpatch::apply_all(
        cli.source_dir.into(),
        env::current_dir()?,
        &cli.patch_file,
        cli.rejects_file.as_deref(),
        cli.strip,
        cli.dryrun,
        matcher,
    )?;
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
