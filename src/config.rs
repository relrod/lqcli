use serde::Deserialize;
use std::fmt::Display;
use tabled::Tabled;

const DEFAULT_REQUEST_DELAY: u64 = 5;
const DEFAULT_POSTPROCESSING_PROMPT: &str = "\
You are editing the transcript for a podcast or video.
You must NEVER modify the content of the transcript.
NEVER summarize the transcript or shorten it.
ALWAYS retain all content.
You must NEVER translate the transcript into any other language.
You MUST ALWAYS produce the original language.
NEVER change what anyone said.
You are responsible for post-processing the transcript to make it more readable.
This includes fixing punctuation, capitalization, and spelling mistakes.
You MAY also add MINOR additional information to the transcript, \
such as the names of speakers, as they speak, if known.
You SHALL NOT add any information that is not present in the transcript.
You SHALL group sentences into paragraphs if and when necessary.
IF the transcript has multiple people, then you shall group sentences into \
paragraphs by speaker.
You SHALL insert a blank line between paragraphs.";
const DEFAULT_POSTPROCESSING_MODEL: &str = "gpt-4o-mini";
const DEFAULT_WHISPER_MODEL: &str = "whisper-1";
const DEFAULT_CONTENT_TYPE: SourceContentType = SourceContentType::Podcast;
const DEFAULT_DOWNLOAD_METHOD: &str = "yt-dlp";
const DEFAULT_TRANSCRIPT_VIA: &str = "openai";

#[derive(Deserialize)]
pub struct LqcliConfig {
    /// Setting specific to the LingQ API
    pub lingq: LingqConfig,

    /// Settings for OpenAI
    pub openai: OpenaiConfig,

    /// Sources are different ways of consuming content such as via RSS feeds
    /// or websites to scrape.
    pub sources: Vec<Source>,
}

#[derive(Deserialize)]
pub struct LingqConfig {
    /// Your LingQ API key
    ///
    /// You can find this at https://www.lingq.com/en/accounts/apikey/
    pub api_key: String,

    /// Time in between requests to the LingQ API (in seconds)
    ///
    /// Used to play nice with the LingQ servers and not hammer them
    /// when importing multiple lessons.
    #[serde(default = "default_request_delay")]
    pub request_delay: u64,
}

#[derive(Deserialize)]
pub struct OpenaiConfig {
    /// Your OpenAI API key
    ///
    /// You can find this at https://platform.openai.com/account/api-keys
    /// or https://platform.openai.com/settings/organization/api-keys for
    /// organization keys or
    /// https://platform.openai.com/settings/profile/api-keys for legacy user
    /// keys.
    pub api_key: String,

    /// Prompt for post-processing a transcript before it is imported.
    ///
    /// This is a GPT prompt sent to whichever model has been selected in
    /// openai.postprocessing_model. The default prompt is a set of
    /// guidelines for post-processing a transcript. This can also be
    /// customized per-fetcher by setting postprocessing_prompt in the
    /// fetcher's configuration.
    #[serde(default = "default_postprocessing_prompt")]
    pub postprocessing_prompt: String,

    /// The model to use for post-processing the transcript
    ///
    /// Valid values can be found at https://platform.openai.com/docs/models.
    /// The default is "gpt-4o-mini".
    #[serde(default = "default_postprocessing_model")]
    pub postprocessing_model: String,

    /// The Whisper model to use for creating transcripts from audio.
    ///
    /// This currently uses the OpenAI API, but in the future will allow for
    /// local processing using the open source Whisper models and whisper-rs.
    /// Defaults to "whisper-1".
    #[serde(default = "default_whisper_model")]
    pub whisper_model: String,
}

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

fn default_request_delay() -> u64 {
    DEFAULT_REQUEST_DELAY
}

fn default_postprocessing_prompt() -> String {
    DEFAULT_POSTPROCESSING_PROMPT.to_string()
}

fn default_postprocessing_model() -> String {
    DEFAULT_POSTPROCESSING_MODEL.to_string()
}

fn default_whisper_model() -> String {
    DEFAULT_WHISPER_MODEL.to_string()
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

impl LqcliConfig {
    pub fn read(path: &str) -> Result<Self, std::io::Error> {
        let normalized_path = shellexpand::tilde(path).to_string();
        let toml = std::fs::read_to_string(normalized_path)?;
        toml::from_str(&toml).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn exists(path: &str) -> bool {
        let normalized_path = shellexpand::tilde(path).to_string();
        std::path::Path::new(&normalized_path).exists()
    }

    pub fn filtered_sources(&self, tags: &[String]) -> Vec<&Source> {
        if tags.is_empty() {
            return self.sources.iter().collect();
        }
        self.sources.iter().filter(|source| {
            if let Some(source_tags) = &source.tags.0 {
                source_tags.iter().any(|tag| tags.contains(tag))
            } else {
                false
            }
        }).collect()
    }
}
