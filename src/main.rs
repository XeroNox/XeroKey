
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::io::{Cursor, Read, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const WIDTH: usize = 54;
const MAX_SUPPORTED_LEN: u32 = 12;

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
    let _ = std::io::stdout().flush();
}

fn print_banner() {
    println!();
    println!("{}", " ██╗  ██╗ ███████╗ ██████╗  ██████╗ ".bright_cyan().bold());
    println!("{}", " ╚██╗██╔╝ ██╔════╝ ██╔══██╗██╔═══██╗".bright_cyan().bold());
    println!("{}", "  ╚███╔╝  █████╗   ██████╔╝██║   ██║".bright_cyan().bold());
    println!("{}", "  ██╔██╗  ██╔══╝   ██╔══██╗██║   ██║".bright_cyan().bold());
    println!("{}", " ██╔╝ ██╗ ███████╗ ██║  ██║╚██████╔╝".bright_cyan().bold());
    println!("{}", " ╚═╝  ╚═╝ ╚══════╝ ╚═╝  ╚═╝ ╚═════╝ ".bright_cyan().bold());
    println!();
    println!("{}", "       X E R O K E Y".bright_white().bold());
    println!("{}", "    ZIP PASSWORD RECOVERY".dimmed());
    println!("{}", "  ──────────────────────────".bright_cyan());
    println!("{}", "  Fast  •  Parallel  •  Local".dimmed());
    println!();
}

fn print_file_info(path: &str, bytes: usize) {
    println!(
        " {} {:<14} {}",
        "▸".bright_cyan(),
        "Archive".dimmed(),
        path.bright_white().bold()
    );
    println!(
        " {} {:<14} {} bytes",
        "▸".bright_cyan(),
        "Size".dimmed(),
        bytes.to_string().yellow().bold()
    );
    println!("{}", " ──────────────────────────────────────────────────".dimmed());
}

fn print_wordlist_info(total: usize, threads: usize) {
    println!(
        " {} {:<14} {}",
        "▸".bright_cyan(),
        "Candidates".dimmed(),
        total.to_string().yellow().bold()
    );
    println!(
        " {} {:<14} {}",
        "▸".bright_cyan(),
        "Workers".dimmed(),
        threads.to_string().green().bold()
    );
    println!("{}", " ──────────────────────────────────────────────────".dimmed());
}

fn print_auto_info(min: u32, max: u32, total: u64, threads: usize) {
    println!(
        " {} {:<14} {}",
        "▸".bright_cyan(),
        "Mode".dimmed(),
        "AUTO-GENERATE".green().bold()
    );
    println!(
        " {} {:<14} {}-{} digits",
        "▸".bright_cyan(),
        "Range".dimmed(),
        min.to_string().yellow().bold(),
        max.to_string().yellow().bold()
    );
    println!(
        " {} {:<14} {}",
        "▸".bright_cyan(),
        "Candidates".dimmed(),
        total.to_string().yellow().bold()
    );
    println!(
        " {} {:<14} {}",
        "▸".bright_cyan(),
        "Workers".dimmed(),
        threads.to_string().green().bold()
    );
    println!("{}", " ──────────────────────────────────────────────────".dimmed());
}

fn create_progress_ui(total: u64) -> ProgressBar {
    let bar = ProgressBar::new(total);
    bar.set_style(
        ProgressStyle::with_template(
            " {spinner:.cyan.bold} {msg}\n {bar:40.cyan/blue} {percent:>3}%\n {pos:>7}/{len:<7} [{elapsed_precise} | {per_sec} | ETA {eta_precise}]",
        )
        .unwrap()
        .progress_chars("━━╸─"),
    );
    bar.enable_steady_tick(Duration::from_millis(100));
    bar
}

fn print_summary(
    found: bool,
    _started: u64,
    completed: u64,
    elapsed: f64,
    password: Option<String>,
) {
    println!();

    let line = format!(" ┌{}┐", "─".repeat(WIDTH - 4));
    let bottom = format!(" └{}┘", "─".repeat(WIDTH - 4));

    let speed = if elapsed > 0.0 {
        completed as f64 / elapsed
    } else {
        0.0
    };

    let speed_display = if speed >= 1_000_000.0 {
        format!("{:.2}M/s", speed / 1_000_000.0)
    } else if speed >= 1_000.0 {
        format!("{:.2}K/s", speed / 1_000.0)
    } else {
        format!("{:.0}/s", speed)
    };

    if found {
        println!("{}", line.green().bold());
        println!(
            " │ {} {:<width$} │",
            "✔".green().bold(),
            "PASSWORD FOUND".green().bold(),
            width = WIDTH - 10
        );

        if let Some(p) = password {
            let display = if p.len() > 30 {
                format!("{}...", &p[..27])
            } else {
                p
            };
            println!(
                " │ {} {:<width$} │",
                "Password".dimmed(),
                display.bright_green().bold(),
                width = WIDTH - 17
            );
        }

        println!(
            " │ {} {:<width$} │",
            "Avg. speed".dimmed(),
            speed_display.cyan().bold(),
            width = WIDTH - 17
        );
        println!(
            " │ {} {:<width$} │",
            "Checked".dimmed(),
            completed.to_string().yellow().bold(),
            width = WIDTH - 17
        );
        println!(
            " │ {} {:<width$} │",
            "Duration".dimmed(),
            format!("{elapsed:.2}s"),
            width = WIDTH - 17
        );
        println!("{}", bottom.green().bold());
        std::process::exit(0);
    } else {
        println!("{}", line.red().bold());
        println!(
            " │ {} {:<width$} │",
            "✘".red().bold(),
            "NO MATCH FOUND".red().bold(),
            width = WIDTH - 10
        );
        println!(
            " │ {} {:<width$} │",
            "Avg. speed".dimmed(),
            speed_display.cyan().bold(),
            width = WIDTH - 17
        );
        println!(
            " │ {} {:<width$} │",
            "Checked".dimmed(),
            completed.to_string().yellow().bold(),
            width = WIDTH - 17
        );
        println!(
            " │ {} {:<width$} │",
            "Duration".dimmed(),
            format!("{elapsed:.2}s"),
            width = WIDTH - 17
        );
        println!("{}", bottom.red().bold());
        std::process::exit(1);
    }
}

#[derive(Debug)]
struct Config {
    zip_path: String,
    mode: Mode,
}

#[derive(Debug)]
enum Mode {
    Wordlist(String),
    Auto { min: u32, max: u32 },
}

fn parse_args() -> Config {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        usage();
    }

    let zip_path = args[0].clone();

    if args.len() >= 2 && args[1] == "--auto" {
        let mut min = 1u32;
        let mut max = 8u32;
        let mut i = 2;

        while i < args.len() {
            match args[i].as_str() {
                "--min-length" => {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("--min-length needs value");
                        usage();
                    }
                    min = args[i].parse().unwrap_or_else(|_| {
                        eprintln!("Invalid --min-length");
                        std::process::exit(2);
                    });
                }
                "--max-length" => {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("--max-length needs value");
                        usage();
                    }
                    max = args[i].parse().unwrap_or_else(|_| {
                        eprintln!("Invalid --max-length");
                        std::process::exit(2);
                    });
                }
                _ => {
                    eprintln!("Unknown arg: {}", args[i]);
                    usage();
                }
            }
            i += 1;
        }

        if min == 0 || max == 0 {
            eprintln!("Length must be >=1");
            std::process::exit(2);
        }
        if min > max {
            eprintln!("min-length {} > max-length {}", min, max);
            std::process::exit(2);
        }
        if max > MAX_SUPPORTED_LEN {
            eprintln!(
                "max-length > {} not supported (would overflow)",
                MAX_SUPPORTED_LEN
            );
            std::process::exit(2);
        }

        Config {
            zip_path,
            mode: Mode::Auto { min, max },
        }
    } else if args.len() >= 2 {
        Config {
            zip_path,
            mode: Mode::Wordlist(args[1].clone()),
        }
    } else {
        usage();
    }
}

fn checked_total(min: u32, max: u32) -> u64 {
    let mut total = 0u64;

    for l in min..=max {
        let pow = 10_u64.checked_pow(l).unwrap_or_else(|| {
            eprintln!("Overflow at 10^{} - use smaller max", l);
            std::process::exit(2);
        });

        total = total.checked_add(pow).unwrap_or_else(|| {
            eprintln!("Total candidates overflow u64 - range too large");
            std::process::exit(2);
        });
    }

    total
}

fn main() {
    let config = parse_args();
    clear_screen();
    print_banner();

    let zip_bytes = Arc::new(std::fs::read(&config.zip_path).unwrap_or_else(|e| {
        eprintln!("Failed to read ZIP: {e}");
        std::process::exit(2);
    }));

    print_file_info(&config.zip_path, zip_bytes.len());

    let mut meta = zip::ZipArchive::new(Cursor::new(zip_bytes.as_slice()))
        .unwrap_or_else(|e| {
            eprintln!("Invalid ZIP: {e}");
            std::process::exit(2);
        });

    let mut encrypted_indices = Vec::new();

    for i in 0..meta.len() {
        if let Ok(entry) = meta.by_index_raw(i) {
            if entry.encrypted() {
                encrypted_indices.push(i);
            }
        }
    }

    if encrypted_indices.is_empty() {
        eprintln!("No encrypted entries.");
        std::process::exit(2);
    }

    println!(
        " {} {:<14} {}",
        "▸".bright_cyan(),
        "Encrypted".dimmed(),
        encrypted_indices.len().to_string().yellow().bold()
    );
    drop(meta);

    let encrypted_arc = Arc::new(encrypted_indices);
    let start = Instant::now();
    let found = Arc::new(AtomicBool::new(false));
    let started = Arc::new(AtomicU64::new(0));
    let completed = Arc::new(AtomicU64::new(0));
    let found_password = Arc::new(Mutex::new(None::<String>));
    let error_flag = Arc::new(AtomicBool::new(false));

    match config.mode {
        Mode::Auto { min, max } => {
            let total = checked_total(min, max);

            if total == 0 {
                eprintln!("No candidates - invalid range");
                std::process::exit(2);
            }

            print_auto_info(min, max, total, rayon::current_num_threads());
            let bar = create_progress_ui(total);

            for len in min..=max {
                if found.load(Ordering::Relaxed) || error_flag.load(Ordering::Relaxed) {
                    break;
                }

                let count = 10_u64.pow(len);

                (0..count).into_par_iter().for_each_init(
                    || {
                        let archive_res =
                            zip::ZipArchive::new(Cursor::new(zip_bytes.as_slice()));
                        (archive_res, Vec::<u8>::with_capacity(8192))
                    },
                    |(archive_res, buffer), num| {
                        if found.load(Ordering::Relaxed)
                            || error_flag.load(Ordering::Relaxed)
                        {
                            return;
                        }

                        let cur_started =
                            started.fetch_add(1, Ordering::Relaxed) + 1;
                        if cur_started % 1024 == 0 {
                            bar.set_position(cur_started);
                        }

                        let archive = match archive_res {
                            Ok(a) => a,
                            Err(e) => {
                                if !error_flag.swap(true, Ordering::SeqCst) {
                                    eprintln!("\nWorker failed to open archive: {}", e);
                                    bar.finish_and_clear();
                                }
                                return;
                            }
                        };

                        let pwd_str = format!("{:0width$}", num, width = len as usize);
                        let mut valid = true;

                        for &idx in encrypted_arc.iter() {
                            let mut file =
                                match archive.by_index_decrypt(idx, pwd_str.as_bytes()) {
                                    Ok(f) => f,
                                    Err(_) => {
                                        valid = false;
                                        break;
                                    }
                                };

                            buffer.clear();

                            if file.read_to_end(buffer).is_err() {
                                valid = false;
                                break;
                            }

                            if file.crc32() != crc32fast::hash(buffer) {
                                valid = false;
                                break;
                            }
                        }

                        completed.fetch_add(1, Ordering::Relaxed);

                        if valid && !found.swap(true, Ordering::SeqCst) {
                            *found_password.lock().unwrap() = Some(pwd_str);
                            bar.finish_and_clear();
                        }
                    },
                );
            }

            bar.finish_and_clear();

            if error_flag.load(Ordering::Relaxed) {
                std::process::exit(3);
            }
        }

        Mode::Wordlist(path) => {
            let dict_file = std::fs::File::open(&path).unwrap_or_else(|e| {
                eprintln!("Failed to open wordlist: {e}");
                std::process::exit(2);
            });

            let mmap = unsafe {
                memmap2::Mmap::map(&dict_file).unwrap_or_else(|e| {
                    eprintln!("Failed to map: {e}");
                    std::process::exit(2);
                })
            };

            let passwords: Vec<&[u8]> = mmap
                .split(|&b| b == b'\n')
                .map(trim_bytes)
                .filter(|s| !s.is_empty())
                .collect();

            if passwords.is_empty() {
                eprintln!("Wordlist empty.");
                std::process::exit(2);
            }

            print_wordlist_info(passwords.len(), rayon::current_num_threads());
            let bar = create_progress_ui(passwords.len() as u64);

            passwords.par_iter().for_each_init(
                || {
                    (
                        zip::ZipArchive::new(Cursor::new(zip_bytes.as_slice())),
                        Vec::<u8>::with_capacity(8192),
                    )
                },
                |(archive_res, buffer), pwd| {
                    if found.load(Ordering::Relaxed)
                        || error_flag.load(Ordering::Relaxed)
                    {
                        return;
                    }

                    started.fetch_add(1, Ordering::Relaxed);

                    let archive = match archive_res {
                        Ok(a) => a,
                        Err(e) => {
                            if !error_flag.swap(true, Ordering::SeqCst) {
                                eprintln!("\nWorker failed to open archive: {}", e);
                                bar.finish_and_clear();
                            }
                            return;
                        }
                    };

                    let mut valid = true;

                    for &idx in encrypted_arc.iter() {
                        let mut file = match archive.by_index_decrypt(idx, pwd) {
                            Ok(f) => f,
                            Err(_) => {
                                valid = false;
                                break;
                            }
                        };

                        buffer.clear();

                        if file.read_to_end(buffer).is_err() {
                            valid = false;
                            break;
                        }

                        if file.crc32() != crc32fast::hash(buffer) {
                            valid = false;
                            break;
                        }
                    }

                    let comp = completed.fetch_add(1, Ordering::Relaxed) + 1;

                    if comp % 512 == 0 {
                        bar.set_position(comp);
                    }

                    if valid && !found.swap(true, Ordering::SeqCst) {
                        *found_password.lock().unwrap() =
                            Some(String::from_utf8_lossy(pwd).to_string());
                        bar.finish_and_clear();
                    }
                },
            );

            bar.finish_and_clear();

            if error_flag.load(Ordering::Relaxed) {
                std::process::exit(3);
            }
        }
    }

    print_summary(
        found.load(Ordering::Relaxed),
        started.load(Ordering::Relaxed),
        completed.load(Ordering::Relaxed),
        start.elapsed().as_secs_f64(),
        found_password.lock().unwrap().clone(),
    );
}

fn usage() -> ! {
    eprintln!("XeroKey v{} - ZIP Password Recovery", env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  xerokey <archive.zip> <wordlist.txt>");
    eprintln!("  xerokey <archive.zip> --auto [--min-length N] [--max-length N]");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  xerokey secret.zip rockyou.txt");
    eprintln!("  xerokey secret.zip --auto --max-length 6");
    eprintln!("  xerokey secret.zip --auto --min-length 4 --max-length 8");
    eprintln!();
    eprintln!("Exit codes: 0=found, 1=not found, 2=bad args, 3=worker error");
    std::process::exit(2);
}

#[inline]
fn trim_bytes(s: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = s.len();

    while start < end && matches!(s[start], b'\r' | b' ' | b'\t') {
        start += 1;
    }

    while end > start && matches!(s[end - 1], b'\r' | b' ' | b'\t' | b'\0') {
        end -= 1;
    }

    &s[start..end]
}
