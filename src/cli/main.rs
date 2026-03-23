use clap::Parser;
use husk::options::{Options, ThousandsSeparator};
use husk::recursive::format_recursive;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "husk", about = "A fast beancount file formatter")]
struct Cli {
    /// Input file(s). Use - for stdin.
    #[arg(default_value = "-")]
    files: Vec<String>,

    /// Indent string
    #[arg(long, default_value = "    ")]
    indent: String,

    /// Column for currency alignment
    #[arg(long, default_value_t = 70)]
    currency_column: usize,

    /// Column for cost/price alignment
    #[arg(long, default_value_t = 75)]
    cost_column: usize,

    /// Thousands separator handling
    #[arg(long, value_enum, default_value_t = ThousandsSeparator::Keep)]
    thousands: ThousandsSeparator,

    /// Add spaces inside cost braces
    #[arg(long)]
    spaces_in_braces: bool,

    /// Disable CJK double-width alignment
    #[arg(long)]
    no_fixed_cjk_width: bool,

    /// Sort entries by date
    #[arg(long)]
    sort: bool,

    /// Recursively format included files
    #[arg(long)]
    recursive: bool,

    /// Write output back to file (in-place)
    #[arg(short = 'w', long)]
    write: bool,
}

fn main() {
    let cli = Cli::parse();

    let options = Options {
        indent: cli.indent.clone(),
        currency_column: cli.currency_column,
        cost_column: cli.cost_column,
        thousands_separator: cli.thousands,
        spaces_in_braces: cli.spaces_in_braces,
        fixed_cjk_width: !cli.no_fixed_cjk_width,
        sort: cli.sort,
    };

    for file in &cli.files {
        if file == "-" {
            let mut input = String::new();
            io::stdin()
                .read_to_string(&mut input)
                .expect("Failed to read stdin");
            let output = husk::format(&input, &options);
            print!("{}", output);
        } else {
            let path = PathBuf::from(file);
            if cli.recursive {
                let results = format_recursive(&path, &options);
                let multi = results.len() > 1;
                for result in results {
                    if cli.write {
                        if let Err(e) = fs::write(&result.path, &result.content) {
                            eprintln!("Error writing {}: {}", result.path.display(), e);
                            process::exit(1);
                        }
                    } else {
                        if multi {
                            println!("==> {} <==", result.path.display());
                        }
                        print!("{}", result.content);
                    }
                }
            } else {
                let input = fs::read_to_string(&path).unwrap_or_else(|e| {
                    eprintln!("Error reading {}: {}", path.display(), e);
                    process::exit(1);
                });
                let output = husk::format(&input, &options);
                if cli.write {
                    if let Err(e) = fs::write(&path, &output) {
                        eprintln!("Error writing {}: {}", path.display(), e);
                        process::exit(1);
                    }
                } else {
                    print!("{}", output);
                }
            }
        }
    }
}
