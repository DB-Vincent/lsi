mod utils;

use std::cmp::Reverse;
use std::fs;
use std::path::{Path};
use std::error::Error;
use std::process;
use std::os::unix::fs::{MetadataExt, PermissionsExt};

use clap::{Parser, ValueEnum};
use chrono::{DateTime, Local};
use users::{get_user_by_uid, get_group_by_gid};
use colored::*;

/// A *very* simple improvement on the already great ls-command.
#[derive(Parser)]
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
    /// Sort the resulting files and/or direcories in a specific way
    #[clap(short, long, value_enum, default_value="name")]
    sort: SortingKey,
    /// Reverse the sorting result
    #[clap(short, long)]
    reverse: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum SortingKey {
    Size,
    Name,
}

fn main() {
    let opts: Opts = Opts::parse();

    if let Err(ref e) = run(opts) {
        println!("{}", e);
        process::exit(1);
    }
}

fn run(opts: Opts) -> Result<(), Box<dyn Error>> {
    let dir = Path::new(&opts.path);

    if dir.is_dir() {
        let mut files: Vec<_> = fs::read_dir(dir).unwrap()
            .into_iter()
            .filter(|r| r.is_ok()) // Get rid of Err variants for Result<DirEntry>
            .map(|r| r.unwrap().path()) // This is safe, since we only have the Ok variants
            .collect();

        match opts.sort {
            SortingKey::Size => {
                if opts.reverse { files.sort_by_key(|file| Reverse(file.metadata().expect("REASON").len())); }
                else { files.sort_by_key(|file| file.metadata().expect("REASON").len()); }
            }
            SortingKey::Name => {
                if opts.reverse { files.sort_by_key(|file| Reverse(file.to_path_buf())); }
                else { files.sort_by_key(|file| file.to_path_buf()); }
            }
        }

        for entry in files {
            let metadata = entry.metadata()?;
            let file_name: &str = entry
                .as_path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();

            if (opts.all || !file_name.starts_with('.')) && (!opts.dirs_only || entry.is_dir()) && (!opts.files_only || !entry.is_dir()) {
                let size = metadata.len();
                let modified: DateTime<Local> = DateTime::from(metadata.modified()?);
                let mode = metadata.permissions().mode();
                let mode_short = format!("{:o}", mode);
                let user = metadata.uid();
                let group = metadata.gid();
                let file_name_colored;

                if entry.is_dir() {
                    if file_name.starts_with(".") {
                        file_name_colored = file_name.blue();
                    } else {
                        file_name_colored = file_name.blue().bold();
                    }
                } else {
                    if file_name.starts_with(".") {
                        file_name_colored = file_name.green();
                    } else {
                        file_name_colored = file_name.green().bold();
                    }
                };

                println!(
                    "{} ({})   {}:{}   {}   {:>9}   {}",
                    utils::parse_permissions(mode as u32),
                    mode_short[mode_short.len() - 3..].to_string(),
                    get_user_by_uid(user).unwrap().name().to_string_lossy(),
                    get_group_by_gid(group).unwrap().name().to_string_lossy(),
                    modified.format("%_d %b %H:%M:%S").to_string(),
                    utils::convert(size as f64),
                    file_name_colored
                );
            }
        }
    }

    Ok(())
}