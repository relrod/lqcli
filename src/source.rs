use crate::config;
use reqwest::Client;
use rss::Item;
use serde::Deserialize;
use std::fmt::Display;
use tabled::Tabled;

const DEFAULT_CONTENT_TYPE: SourceContentType = SourceContentType::Podcast;
const DEFAULT_DOWNLOAD_METHOD: &str = "yt-dlp";
const DEFAULT_TRANSCRIPT_VIA: &str = "openai";

#[derive(Deserialize)]
#[serde(transparent)]
pub struct Tags(pub Option<Vec<String>>);

#[derive(Deserialize, Tabled)]
pub struct Source {
    /// Content type
    ///
    /// This describes how to find the audio content for this source.
    /// For example, a value of "podcast" would mean that lqcli needs to look
    /// at the RSS feed for <enclosure> tags. "youtube" would mean that lqcli
    /// is looking at the YouTube's RSS feed for videos, and so on.
    ///
    /// Current possible values are: podcast (default), youtube
    #[serde(default = "default_content_type")]
    pub content_type: SourceContentType,

    /// Download method
    ///
    /// Once the audio or video link has been identified (using the
    /// content_type), this describes how to download the content.
    /// A safe bet for many content types is "yt-dlp", which can handle
    /// many different types of content (not just YouTube). This is the
    /// default.
    #[serde(default = "default_download_method")]
    pub download_method: String,

    /// The URL containing to the feed or page to scrape
    #[tabled(skip)]
    pub url: String,

    /// The name of the fetcher, mostly just for display purposes on the CLI
    #[tabled(order = 0)]
    pub name: String,

    /// The prompt to use for post-processing this fetcher's content
    /// Defaults to openai.postprocessing_prompt.
    #[tabled(skip)]
    pub postprocessing_prompt: Option<String>,

    /// The course ID to create a lesson in for each fetched item from this
    /// source.
    pub course_id: u64,

    /// The two-letter language code. The LingQ API uses this because course IDs
    /// are unique per language.
    pub language: String,

    /// Tags allow you to group sources. One place where this could be useful
    /// is to tag sources that are known to update daily vs multiple times a
    /// day. Then you could set up two automations, one that runs daily and
    /// one that runs every 6 hours, each passing in the appropriate tag.
    /// This allows for speeding up runs by not having to check every source
    /// every time.
    pub tags: Tags,

    /// Transcripts are normally assumed to be created by the OpenAI Whisper
    /// model described in openai.whisper_model. But sometimes, some sources
    /// might need special handling. For example the Easy German videos have
    /// transcripts available for members and lqcli knows how to download them.
    /// In this case, you would set this to "easy-german" or
    /// "super-easy-german". The default is "openai". You can also set to
    /// "lingq". LingQ will use Whisper (which is cheaper for you, the user,
    /// than using OpenAI), but it doesn't do any post-processing. This is
    /// normally good enough for single-speaker content.
    #[serde(default = "default_transcript_via")]
    pub transcript_via: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceContentType {
    Podcast,
    Youtube,
}

impl Display for Tags {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.0 {
            Some(tags) => write!(f, "{}", tags.join(", ")),
            None => write!(f, ""),
        }?;
        Ok(())
    }
}

impl Display for SourceContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SourceContentType::Podcast => write!(f, "podcast"),
            SourceContentType::Youtube => write!(f, "youtube"),
        }
    }
}

fn default_content_type() -> SourceContentType {
    DEFAULT_CONTENT_TYPE
}

fn default_download_method() -> String {
    DEFAULT_DOWNLOAD_METHOD.to_string()
}

fn default_transcript_via() -> String {
    DEFAULT_TRANSCRIPT_VIA.to_string()
}

pub fn get_audio_link(source: &Source, item: &Item) -> Option<String> {
    match source.content_type {
        SourceContentType::Podcast => {
            item.enclosure.as_ref().map(|enclosure| enclosure.url.clone())
        }
        SourceContentType::Youtube => {
            item.link.clone()
        }
    }
}
