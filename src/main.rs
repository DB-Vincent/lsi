mod utils;

use std::fs;
use std::path::{Path, PathBuf};
use std::error::Error;
use std::process;
use std::os::unix::fs::PermissionsExt;

use clap::Parser;
use chrono::{DateTime, Local};

/// A *very* simple improvement on the already great ls-command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Opts {
    /// Path to the directory
    #[clap(value_parser, default_value=".")]
    path: String,
    /// Show all files, including hidden ones
    #[clap(short, long)]
    all: bool,
    /// Show only directories
    #[clap(short, long)]
    dirs_only: bool,
    /// Show only files
    #[clap(short, long)]
    files_only: bool,
}

fn main() {
    let opts: Opts = Opts::parse();

    if let Err(ref e) = run(Path::new(&opts.path), opts.all, opts.dirs_only, opts.files_only) {
        println!("{}", e);
        process::exit(1);
    }
}

fn run(dir: &Path, all: bool, dirs_only: bool, files_only: bool) -> Result<(), Box<dyn Error>> {
    if dir.is_dir() {
        let files: Result<Vec<PathBuf>, Box<dyn Error>>= Ok(fs::read_dir(dir)?
            .into_iter()
            .filter(|r| r.is_ok()) // Get rid of Err variants for Result<DirEntry>
            .map(|r| r.unwrap().path()) // This is safe, since we only have the Ok variants
            .collect());

        for entry in files? {
            let metadata = entry.metadata()?;
            let file_name: &str = entry
                .as_path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();
            let size = metadata.len();
            let modified: DateTime<Local> = DateTime::from(metadata.modified()?);
            let mode = metadata.permissions().mode();

            if (all || !file_name.starts_with('.')) && (!dirs_only || entry.is_dir()) && (!files_only || !entry.is_dir()) {
                println!(
                    "{}   {}   {:>9}   {}",
                    utils::parse_permissions(mode as u32),
                    modified.format("%_d %b %H:%M:%S").to_string(),
                    utils::convert(size as f64),
                    file_name
                );
            }
        }
    }

    Ok(())
}