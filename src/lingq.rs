//! Provides an interface to the LingQ API (or at least the parts we need).

use crate::config;
use reqwest::{Client, header};
use serde::Deserialize;

pub struct LingqClient {
    client: Client,
}

#[derive(Debug, Deserialize)]
pub struct LingqCourse {
    pub pk: u64,
    pub url: String,
    pub title: String,
    pub lessons: Vec<LingqLesson>,
}

#[derive(Debug, Deserialize)]
pub struct LingqLesson {
    pub title: String,
    pub url: String,
}

impl LingqClient {
    pub fn new(lingq_config: &config::LingqConfig) -> Self {
        let mut headers = header::HeaderMap::new();
        let api_key = lingq_config.api_key.as_str();
        headers.insert("Authorization", header::HeaderValue::from_str(&format!("Token {}", api_key)).unwrap());
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        Self { client }
    }

    pub async fn get_lesson_titles(&self, language: &str, course_id: u64) -> Result<Vec<String>, reqwest::Error> {
        let url = format!("https://www.lingq.com/api/v2/{}/collections/{}/", language, course_id);
        let response = self.client.get(&url).send().await?;
        response.error_for_status_ref()?;
        let json: LingqCourse = response.json().await?;
        let lessons = json.lessons;
        let titles: Vec<String> = lessons.into_iter().map(|lesson| lesson.title).collect();
        Ok(titles)
    }

    pub async fn create_lesson(&self, course_id: u64, title: &str, text: &str, mp3: Option<Vec<u8>>) -> Result<(), reqwest::Error> {
        let url = "https://www.lingq.com/api/v3/de/lessons/import/";
        let mut form = reqwest::multipart::Form::new()
            .text("title", title.to_string())
            .text("collection", course_id.to_string())
            .text("save", "true".to_string())
            .text("text", text.to_string());
        if let Some(mp3) = mp3 {
            form = form.part("audio", reqwest::multipart::Part::bytes(mp3).file_name("audio.mp3"));
        }
        let response = self.client.post(url).multipart(form).send().await?;
        response.error_for_status_ref()?;
        Ok(())
    }
}
