use crate::config;
use reqwest::Client;
use rss::Item;

pub fn get_audio_link(source: &config::Source, item: &Item) -> Option<String> {
    match source.content_type {
        config::SourceContentType::Podcast => {
            item.enclosure.as_ref().map(|enclosure| enclosure.url.clone())
        }
        config::SourceContentType::Youtube => {
            item.link.clone()
        }
    }
}
