use clap::Parser;
use owo_colors::OwoColorize;
use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Parser, Clone)]
#[command(name = "kti")]
#[command(about = "A simple tool to correct file extensions to match their file signatures.")]
struct Kti {
    path: Option<PathBuf>,

    #[arg(
        short = 'a',
        short_alias = '.',
        long = "show-hidden",
        help = "Kti will not ignore hidden files"
    )]
    show_hidden: bool,

    #[arg(
        short = 'm',
        long = "max-depth",
        value_name = "INTEGER",
        help = "Sets max depth for kti to reach"
    )]
    max_depth: Option<usize>,

    #[arg(
        short = 'd',
        long = "only-diff",
        help = "Only prints out files with different file extensions"
    )]
    only_different: bool,

    #[arg(
        short = 's',
        long = "silent",
        help = "Does not print out files changed"
    )]
    silent: bool,

    #[arg(short = 'L', long = "follow-links", help = "Follows symbolic links")]
    follow_links: bool,

    #[arg(long = "dry-run", help = "Runs kti without any changes to the files")]
    dry_run: bool,

    #[arg(short = 'c', long = "color", help = "Adds colors to the output.")]
    colored: bool,
}

fn main() {
    let kti = Kti::parse();
    let root_path = kti.path.clone().unwrap_or(PathBuf::from("."));

    if let Ok(exists) = fs::exists(&root_path) {
        if !exists {
            eprintln!("Path does not exist.")
        }
        let mut walkdir = WalkDir::new(&root_path);

        if let Some(depth) = kti.max_depth {
            walkdir = walkdir.max_depth(depth)
        }

        if kti.follow_links {
            walkdir = walkdir.follow_links(true)
        }

        let entries = walkdir.into_iter();

        let mut diff_counter = 0;
        for entry_result in entries.filter_entry(|e| filter_entries(e, &kti)) {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("Error reading entry: {}", e);
                    continue;
                }
            };

            if !entry.path().is_file() {
                continue;
            }

            let current_extension: String = match entry.path().extension() {
                Some(ext) => ext.to_string_lossy().to_string(),
                None => {
                    if kti.colored {
                        "No extension".yellow().to_string()
                    } else {
                        "No extension".to_string()
                    }
                }
            };

            let detected_extension: String = match get_correct_extension(entry.path()) {
                Ok(Some(ext)) => ext,
                Ok(None) => {
                    if kti.colored {
                        "Not detected".yellow().to_string()
                    } else {
                        "Not detected".to_string()
                    }
                }
                Err(e) => e.to_string(),
            };

            let file_name = entry.file_name();

            let file_path = entry.path();

            if different_extensions(&current_extension, &detected_extension) {
                diff_counter += 1;
            }
            if kti.colored {
                print_colored_report(
                    &file_name.to_string_lossy(),
                    &file_path.to_string_lossy(),
                    &kti,
                    &current_extension,
                    &detected_extension,
                );
            } else {
                print_report(
                    &file_name.to_string_lossy(),
                    &file_path.to_string_lossy(),
                    &kti,
                    &current_extension,
                    &detected_extension,
                );
            }

            if !kti.dry_run && different_extensions(&current_extension, &detected_extension) {
                let mut updated_path = file_path.to_path_buf();
                updated_path.set_extension(detected_extension);

                match fs::rename(file_path, &updated_path) {
                    Ok(_) => {
                        println!("{:?} -> {:?}", file_path, updated_path);
                    }
                    Err(e) => {
                        eprintln!("Could not rename file.");
                        eprintln!("{}", e)
                    }
                };
            }
        }
        println!("Differences found: {}", diff_counter);
    } else {
        println!("Failed reading directory")
    }
}

fn print_colored_report(name: &str, path: &str, kti: &Kti, current: &str, detected: &str) {
    if !kti.silent && !kti.only_different && !different_extensions(current, detected) {
        println!();
        println!("Path: {}", path.bright_green());
        println!("Name: {}", name.bright_green());
        println!("Current:  {}", current.bright_green());
        println!("Detected: {}", detected.bright_green());
    }
    if !kti.silent && !kti.only_different && different_extensions(current, detected) {
        println!();
        println!("Path: {}", path.bright_green());
        println!("Name: {}", name.bright_green());
        println!("Current:  {}", current.bright_red());
        println!("Detected: {}", detected.bright_green());
    }
    if !kti.silent && kti.only_different && different_extensions(current, detected) {
        println!();
        println!("Path: {}", path.bright_green());
        println!("Name: {}", name.bright_green());
        println!("Current:  {}", current.bright_red());
        println!("Detected: {}", detected.bright_green());
    }
}

fn print_report(name: &str, path: &str, kti: &Kti, current: &str, detected: &str) {
    if !kti.silent {
        if !kti.only_different && !different_extensions(current, detected) {
            println!();
            println!("Path: {}", path);
            println!("Name: {}", name);
            println!("Current:  {}", current);
            println!("Detected: {}", detected);
        }
        if !kti.only_different && different_extensions(current, detected) {
            println!();
            println!("Path: {}", path);
            println!("Name: {}", name);
            println!("Current:  {}", current);
            println!("Detected: {}", detected);
        }
        if kti.only_different && different_extensions(current, detected) {
            println!();
            println!("Path: {}", path);
            println!("Name: {}", name);
            println!("Current:  {}", current);
            println!("Detected: {}", detected);
        }
    }
}

fn get_correct_extension(path: &Path) -> Result<Option<String>, Box<dyn Error>> {
    let mut file = fs::File::open(path)?;
    let mut buffer = [0; 32];
    let bytes_read = file.read(&mut buffer)?;

    let extension = match &buffer[0..std::cmp::min(bytes_read, 32)] {
        [0x47, 0x49, 0x46, 0x38, 0x37, 0x61, ..] | [0x47, 0x49, 0x46, 0x38, 0x39, 0x61, ..] => {
            Some("gif")
        }
        [0xFF, 0xFB, ..] | [0xFF, 0xF3, ..] | [0xFF, 0xF2, ..] | [0x49, 0x44, 0x33, ..] => {
            Some("mp3")
        }
        [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, ..] => Some("png"),
        [0x25, 0x50, 0x44, 0x46, 0x2D, ..] => Some("pdf"),
        [0x4F, 0x67, 0x67, 0x53, ..] => Some("ogg"),
        [0x1A, 0x45, 0xDF, 0xA3, ..] => Some("mkv"),
        [0x66, 0x4C, 0x61, 0x43, ..] => Some("flac"),
        [0xFF, 0xD8, 0xFF, ..] => Some("jpg"),
        buf if buf.len() >= 12 && &buf[0..4] == b"RIFF" => match &buf[8..12] {
            b"WEBP" => Some("webp"),
            b"WAVE" => Some("wav"),
            _ => None,
        },
        buf if buf.len() >= 12 && &buf[4..8] == b"ftyp" => match &buf[8..12] {
            b"qt  " => Some("mov"),
            b"avc1" | b"isom" | b"mmp4" | b"mp41" | b"mp42" | b"mp71" | b"msnv" | b"M4V " => {
                Some("mp4")
            }
            _ => None,
        },
        _ => None,
    };
    let extension = extension.map(|ext| ext.to_string());
    Ok(extension)
}

fn filter_entries(entry: &DirEntry, options: &Kti) -> bool {
    if !options.show_hidden && is_hidden(entry) {
        return false;
    }
    true
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn different_extensions(current: &str, detected: &str) -> bool {
    if detected.contains("No") || detected.contains("Err") {
        return false;
    }
    if current == "jpeg" && detected == "jpg" {
        return false;
    }
    if current == detected {
        return false;
    }
    true
}
