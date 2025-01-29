use atom_syndication::{Feed as AtomFeed, Entry};
use rss::{Channel, Item as RssItem};
use serde::Deserialize;
use std::fmt::Display;
use tabled::Tabled;

use crate::fetch::{DownloadMethod, fetch};

const DEFAULT_CONTENT_TYPE: ContentType = ContentType::Syndication;
const DEFAULT_DOWNLOAD_METHOD: DownloadMethod = DownloadMethod::YtDlp;
const DEFAULT_TRANSCRIPT_VIA: &str = "openai";

#[derive(Deserialize)]
#[serde(transparent)]
pub struct Tags(pub Option<Vec<String>>);

impl Display for Tags {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.0 {
            Some(tags) => write!(f, "{}", tags.join(", ")),
            None => write!(f, ""),
        }?;
        Ok(())
    }
}

#[derive(Deserialize, Tabled)]
pub struct Source {
    /// Content type
    ///
    /// This describes how to find the audio content for this source.
    ///
    /// The default (rss_atom) will use heuristics to determine the content
    /// type. It will try to parse the feed as an RSS feed first. If that fails,
    /// it will try to parse it as an Atom feed. If RSS, it will look for
    /// an enclosure and pull the link out that way. If Atom, it will look
    /// for a link in the entry. In the future, other content types may be
    /// added for special cases where this doesn't work.
    #[serde(default = "default_content_type")]
    pub content_type: ContentType,

    /// Download method
    ///
    /// Once the audio or video link has been identified (using the
    /// content_type), this describes how to download the content.
    /// A safe bet for many content types is "yt-dlp", which can handle
    /// many different types of content (not just YouTube). This is the
    /// default.
    #[serde(default = "default_download_method")]
    pub download_method: DownloadMethod,

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
#[serde(rename_all = "kebab-case")]
pub enum ContentType {
    /// The content comes from some kind of syndication feed (RSS or Atom).
    Syndication,
}

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ContentType::Syndication => write!(f, "Syndication Feed"),
        }
    }
}

#[derive(Debug)]
pub enum SourceError {
    FetchError(reqwest::Error),
    // It would be nice to have an accumulating Result type where we can
    // try multiple parsers and accumulate the errors if all of them fail.
    // TODO.
    ParseError(String),
    AudioDownloadError(std::io::Error),
}

impl From<reqwest::Error> for SourceError {
    fn from(err: reqwest::Error) -> Self {
        SourceError::FetchError(err)
    }
}

impl From<std::io::Error> for SourceError {
    fn from(err: std::io::Error) -> Self {
        SourceError::AudioDownloadError(err)
    }
}

impl Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SourceError::FetchError(err) => write!(f, "Fetch error: {}", err),
            SourceError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SourceError::AudioDownloadError(err) => write!(f, "Audio download error: {}", err),
        }
    }
}

fn default_content_type() -> ContentType {
    DEFAULT_CONTENT_TYPE
}

fn default_download_method() -> DownloadMethod {
    DEFAULT_DOWNLOAD_METHOD
}

fn default_transcript_via() -> String {
    DEFAULT_TRANSCRIPT_VIA.to_string()
}

#[derive(Debug)]
pub struct StaticItem {
    pub url: String,
    pub title: String,
}

#[derive(Debug)]
/// What kind of individual item are we dealing with?
pub enum SourceItem {
    Rss(RssItem),
    Atom(Entry),
    /// Just a static link to an audio file somewhere.
    Static(StaticItem),
}

#[derive(Debug)]
/// A source's feed can represent either an RSS feed or an Atom feed.
pub enum Feed {
    Rss(Channel),
    Atom(AtomFeed),
}

impl Feed {
    /// We don't know if a link is RSS or Atom. So first we try to parse it as
    /// RSS. If that fails, we try to parse it as Atom.
    pub async fn from_source(source: &Source) -> Result<Self, SourceError> {
        let content = reqwest::get(&source.url).await?.bytes().await?;
        rss::Channel::read_from(&content[..])
            .map(Feed::Rss)
            .or_else(|_| {
                atom_syndication::Feed::read_from(&content[..])
                    .map(Feed::Atom)
            })
            .map_err(|_| SourceError::ParseError("Could not parse as RSS or Atom feed".to_string()))
    }

    pub fn items(&self, count: usize) -> Vec<SourceItem> {
        match self {
            Feed::Rss(channel) => channel
                .items
                .iter()
                .take(count)
                .map(|item| SourceItem::Rss(item.clone()))
                .collect(),
            Feed::Atom(feed) => feed
                .entries()
                .iter()
                .take(count)
                .map(|entry| SourceItem::Atom(entry.clone()))
                .collect(),
        }
    }
}

impl SourceItem {
    pub fn from_url_and_title(url: &str, title: &str) -> Self {
        SourceItem::Static(StaticItem { url: url.into(), title: title.into() })
    }

    pub fn get_audio_link(&self) -> Option<String> {
        match self {
            SourceItem::Rss(item) => {
                item.enclosure.as_ref().map(|enclosure| enclosure.url.clone())
            }
            SourceItem::Atom(entry) => {
                entry.links().first().map(|link| link.href().to_string())
            }
            SourceItem::Static(item) => Some(item.url.clone()),
        }
    }

    pub fn title(&self) -> Option<String> {
        match self {
            SourceItem::Rss(item) => item.title.clone(),
            SourceItem::Atom(entry) => Some(entry.title().to_string()),
            SourceItem::Static(item) => Some(item.title.clone()),
        }
    }

    pub async fn download_audio(&self, method: DownloadMethod) -> Result<Vec<u8>, SourceError> {
        fetch(self, method)
    }
}
