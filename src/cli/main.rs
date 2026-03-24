use beanfmt::config::FileConfig;
use beanfmt::options::{SortOrder, SortableDirective, TimelessPosition};
use beanfmt::recursive::format_recursive;
use clap::{ArgAction, Parser};
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "beanfmt", about = "A fast beancount file formatter")]
struct Cli {
    /// Input file(s). Use - for stdin.
    #[arg(default_value = "-")]
    files: Vec<String>,

    /// Number of spaces for indentation
    #[arg(long)]
    indent: Option<usize>,

    /// Column for currency alignment
    #[arg(long)]
    currency_column: Option<usize>,

    /// Column for cost/price alignment
    #[arg(long)]
    cost_column: Option<usize>,

    /// Thousands separator handling (add, remove, keep)
    #[arg(long)]
    thousands: Option<String>,

    /// Add spaces inside cost braces
    #[arg(long, action = ArgAction::SetTrue, overrides_with = "no_spaces_in_braces")]
    spaces_in_braces: bool,

    /// Disable spaces inside cost braces
    #[arg(long = "no-spaces-in-braces", action = ArgAction::SetTrue, overrides_with = "spaces_in_braces", hide = true)]
    no_spaces_in_braces: bool,

    /// Enable CJK double-width alignment
    #[arg(long, action = ArgAction::SetTrue, overrides_with = "no_fixed_cjk_width")]
    fixed_cjk_width: bool,

    /// Disable CJK double-width alignment
    #[arg(long = "no-fixed-cjk-width", action = ArgAction::SetTrue, overrides_with = "fixed_cjk_width", hide = true)]
    no_fixed_cjk_width: bool,

    /// Sort entries by date (asc, desc, off)
    #[arg(long, value_enum, num_args = 0..=1, default_missing_value = "asc")]
    sort: Option<SortOrder>,

    /// Disable sorting entries by date (backwards compat alias for --sort off)
    #[arg(long = "no-sort", action = ArgAction::SetTrue, hide = true)]
    no_sort: bool,

    /// Where to place timeless entries within a day (begin or end)
    #[arg(long, value_enum)]
    sort_timeless: Option<TimelessPosition>,

    /// Directive types to exclude from sorting; excluded directives act as sort barriers (comma-separated)
    #[arg(long, value_enum, value_delimiter = ',')]
    sort_exclude: Option<Vec<SortableDirective>>,

    /// Recursively format included files
    #[arg(long)]
    recursive: bool,

    /// Write output back to file (in-place)
    #[arg(short = 'w', long)]
    write: bool,

    /// Skip loading configuration files
    #[arg(long)]
    no_config: bool,
}

impl Cli {
    fn to_file_config(&self) -> FileConfig {
        let spaces_in_braces = if self.spaces_in_braces {
            Some(true)
        } else if self.no_spaces_in_braces {
            Some(false)
        } else {
            None
        };

        let fixed_cjk_width = if self.fixed_cjk_width {
            Some(true)
        } else if self.no_fixed_cjk_width {
            Some(false)
        } else {
            None
        };

        let sort = if self.no_sort {
            Some(SortOrder::Off)
        } else {
            self.sort
        };

        FileConfig {
            indent: self.indent,
            currency_column: self.currency_column,
            cost_column: self.cost_column,
            thousands: self.thousands.clone(),
            spaces_in_braces,
            fixed_cjk_width,
            sort,
            sort_timeless: self.sort_timeless,
            sort_exclude: self.sort_exclude.clone(),
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let file_config = if cli.no_config {
        FileConfig::default()
    } else {
        let global = beanfmt::config::load_global();
        let cwd = std::env::current_dir().unwrap_or_default();
        let project = beanfmt::config::find_project_config(&cwd);
        global.merge(project)
    };

    let cli_config = cli.to_file_config();
    let options = file_config.merge(cli_config).into_options();

    for file in &cli.files {
        if file == "-" {
            let mut input = String::new();
            io::stdin()
                .read_to_string(&mut input)
                .expect("Failed to read stdin");
            let output = beanfmt::format(&input, &options);
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
                let output = beanfmt::format(&input, &options);
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
