use serde::de::{IntoDeserializer, value};
use serde::Deserialize;
use std::fmt::Display;
use std::io;
use std::io::Read;
use std::fs::File;
use std::process::Command;
use std::str::FromStr;
use tempfile::NamedTempFile;

use crate::source::{Source, SourceItem, SourceError};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DownloadMethod {
    /// `yt-dlp` - Use yt-dlp to download the content.
    YtDlp,
}

impl FromStr for DownloadMethod {
    type Err = value::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::deserialize(s.into_deserializer())
    }
}

impl Display for DownloadMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DownloadMethod::YtDlp => write!(f, "yt-dlp"),
        }
    }
}

/// Call `yt-dlp` to download the content.
///
/// Download the content and return a Vec<u8> with the content.
fn yt_dlp(url: &str) -> io::Result<Vec<u8>> {
    let tmpfile = NamedTempFile::with_suffix(".mp3")?;
    let tmpfile_path = tmpfile.path();
    let output = Command::new("yt-dlp")
        .arg("--format")
        .arg("bestaudio/best")
        .arg("-x")
        .arg("--audio-format")
        .arg("mp3")
        .arg("--output")
        .arg(tmpfile_path)
        .arg("--force-overwrites")
        .arg(url)
        .output()?;
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("yt-dlp failed: {}", String::from_utf8_lossy(&output.stderr)),
        ));
    }
    let mut tmpfile_reopened = File::open(tmpfile_path)?;
    let mut content = Vec::new();
    tmpfile_reopened.read_to_end(&mut content)?;
    Ok(content)
}

pub fn fetch(item: &SourceItem, method: DownloadMethod) -> Result<Vec<u8>, SourceError> {
    let link = item.get_audio_link().unwrap();
    match method {
        DownloadMethod::YtDlp => yt_dlp(&link).map_err(SourceError::from),
    }
}
