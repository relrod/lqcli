mod config;
mod openai;

use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;
use tabled::{Table, settings::Style};

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Green.on_default())
}

#[derive(Parser, Debug)]
#[command(version, styles = styles())]
/// Command-line interface to import content into language-learning platforms
/// such as LingQ.
struct Cli {
    /// Path to the configuration file to create or read from
    #[arg(short, long, default_value = "~/.lqcli.toml")]
    config_file: String,

    /// The category of action to perform
    #[command(subcommand)]
    subcommand: MainSubcommand,
}

#[derive(Debug, Subcommand)]
enum MainSubcommand {
    /// Import content from periodicals such as podcasts or YouTube channels
    #[command(subcommand)]
    Sources(SourcesSubcommand),
}

#[derive(Debug, Subcommand)]
enum SourcesSubcommand {
    /// Synchronize content from sources
    Sync {
        /// Only synchronize sources with these tags
        #[arg(short, long)]
        tags: Option<Vec<String>>,

        /// Don't actually do anything, just list the sources
        #[arg(short, long, default_value = "false")]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // First make sure the configuration file exists
    if !config::LqcliConfig::exists(&cli.config_file) {
        eprintln!("Configuration file {} does not exist", cli.config_file);
        std::process::exit(1);
    }

    // Try to read the configuration file
    let config = match config::LqcliConfig::read(&cli.config_file) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error reading configuration file: {}", e);
            std::process::exit(1);
        }
    };

    match cli.subcommand {
        MainSubcommand::Sources(subcommand) => match subcommand {
            SourcesSubcommand::Sync { tags, dry_run } => {
                // Get the filtered sources by tags
                // source.tags will be a Tags(Option<Vec<String>>)
                let filtered_sources = config.sources.iter().filter(|source| {
                    tags.as_ref()
                        .map(|tags| {
                            source
                                .tags
                                .0
                                .as_ref()
                                .map_or(false, |source_tags| tags.iter().any(|tag| source_tags.contains(tag)))
                        })
                        .unwrap_or(true)
                });
                if dry_run {
                    println!("Would synchronize the following sources:");
                    let mut table = Table::new(filtered_sources);
                    table.with(Style::modern());
                    println!("{}", table);

                    let resp = openai::postprocess(
                        "This is a test.",
                        config.openai.postprocessing_prompt.as_str(),
                        &config.openai
                    ).await.unwrap();
                    println!("{}", resp);
                } else {
                    println!("Synchronizing sources:");
                }
            }
        },
    }
}
