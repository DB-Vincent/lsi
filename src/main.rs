use std::fs;
use std::path::{Path, PathBuf};
use std::error::Error;
use std::process;
use std::cmp;
use std::os::unix::fs::PermissionsExt;
use libc::{S_IRGRP, S_IROTH, S_IRUSR, S_IWGRP, S_IWOTH, S_IWUSR, S_IXGRP, S_IXOTH, S_IXUSR};

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

fn parse_permissions(mode: u32) -> String {
    let user = triplet(mode, S_IRUSR, S_IWUSR, S_IXUSR);
    let group = triplet(mode, S_IRGRP, S_IWGRP, S_IXGRP);
    let other = triplet(mode, S_IROTH, S_IWOTH, S_IXOTH);
    [user, group, other].join("")
}

fn triplet(mode: u32, read: u32, write: u32, execute: u32) -> String {
    match (mode & read, mode & write, mode & execute) {
        (0, 0, 0) => "---",
        (_, 0, 0) => "r--",
        (0, _, 0) => "-w-",
        (0, 0, _) => "--x",
        (_, 0, _) => "r-x",
        (_, _, 0) => "rw-",
        (0, _, _) => "-wx",
        (_, _, _) => "rwx",
    }.to_string()
}

pub fn convert(num: f64) -> String {
    let negative = if num.is_sign_positive() { "" } else { "-" };
    let num = num.abs();
    let units = ["B ", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    if num < 1_f64 {
        return format!("{}{:>6}{}", negative, num, "B");
    }
    let delimiter = 1000_f64;
    let exponent = cmp::min((num.ln() / delimiter.ln()).floor() as i32, (units.len() - 1) as i32);
    let pretty_bytes = format!("{:.2}", num / delimiter.powi(exponent)).parse::<f64>().unwrap() * 1_f64;
    let unit = units[exponent as usize];
    format!("{}{} {}", negative, pretty_bytes, unit)
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
                    parse_permissions(mode as u32),
                    modified.format("%_d %b %H:%M:%S").to_string(),
                    convert(size as f64),
                    file_name
                );
            }
        }
    }

    Ok(())
}