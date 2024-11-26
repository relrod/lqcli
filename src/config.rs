use crate::source;
use serde::Deserialize;

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

#[derive(Deserialize)]
pub struct LqcliConfig {
    /// Setting specific to the LingQ API
    pub lingq: LingqConfig,

    /// Settings for OpenAI
    pub openai: OpenaiConfig,

    /// Sources are different ways of consuming content such as via RSS feeds
    /// or websites to scrape.
    pub sources: Vec<source::Source>,
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

    pub fn filtered_sources(&self, tags: &[String]) -> Vec<&source::Source> {
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
