mod config;
mod openai;
mod lingq;
mod source;

use clap::{
    builder::styling::{AnsiColor, Effects, Styles},
    Parser, Subcommand,
};
use owo_colors::OwoColorize;
use tabled::{
    settings::{
        style::HorizontalLine,
        object::Rows,
        Color, Style,
    },
    Table,
};

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

    /// List sources, possibly filtered by tags
    List {
        /// Only list sources with these tags
        #[arg(short, long)]
        tags: Option<Vec<String>>,
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

    let lingq_client = lingq::LingqClient::new(&config.lingq);

    match cli.subcommand {
        MainSubcommand::Sources(subcommand) => match subcommand {
            SourcesSubcommand::List { tags } => {
                let filtered_sources = config.filtered_sources(&tags.unwrap_or_default());
                let mut table = Table::new(filtered_sources.clone());
                let style = Style::modern()
                    .horizontals([(1, HorizontalLine::inherit(Style::modern()).horizontal('═'))]);
                table.with(style)
                    .modify(Rows::first(), Color::BOLD);
                println!("{}", table);
            }
            SourcesSubcommand::Sync { tags, dry_run } => {
                // Get the filtered sources by tags
                // source.tags will be a Tags(Option<Vec<String>>)
                let filtered_sources = config.filtered_sources(&tags.unwrap_or_default());

                for source in filtered_sources {
                    println!("Syncing source: {}", source.name);

                    let lesson_titles_resp = lingq_client.get_lesson_titles(&source.language, source.course_id).await;
                    let lesson_titles = lesson_titles_resp.unwrap_or_else(|e| {
                        eprintln!("Error getting lesson titles for {}: {}", source.name, e);
                        vec![]
                    });

                    // Latest 5 items (this number should be configurable)
                    let items = match source::SourceFeed::from_source(&source).await {
                        Ok(feed) => feed.items(5),
                        Err(e) => {
                            eprintln!("Error getting items for {}: {}", source.name, e);
                            continue;
                        }
                    };
                    for item in items {
                        // Get the audio url and print it, for now.
                        let audio_link = item.get_audio_link(&source);
                        if let Some(audio_link) = audio_link {
                            println!("{}", audio_link);
                        } else {
                            eprintln!("No audio link found for {}", source.name);
                        }
                    }

                    // let resp = openai::postprocess(
                    //     "hallo das hier ist ein test",
                    //     config.openai.postprocessing_prompt.as_str(),
                    //     &config.openai
                    // ).await.unwrap();
                    // println!("{}", resp);
                }
            }
        },
    }
}
