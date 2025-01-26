/// Use OpenAI to postprocess a transcript.

use crate::config;

use async_openai::{
    types::AudioInput,
    types::CreateChatCompletionRequestArgs,
    types::CreateChatCompletionRequest,
    types::CreateTranscriptionRequestArgs,
    types::CreateTranscriptionRequest,
    types::ChatCompletionRequestSystemMessageArgs,
    types::ChatCompletionRequestUserMessageArgs,
    Client,
    config::OpenAIConfig as LibOpenAIConfig
};

pub struct OpenAI {
    config: config::OpenaiConfig,
    client: Client<LibOpenAIConfig>,
}

impl OpenAI {
    pub fn new(config: config::OpenaiConfig) -> Self {
        let api_key = config.api_key.clone();
        let client_config = LibOpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(client_config);
        Self { config, client }
    }

    pub async fn postprocess(&self, transcript: &str) -> Option<String> {
        let model = self.config.postprocessing_model.clone();
        let request: CreateChatCompletionRequest = CreateChatCompletionRequestArgs::default()
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(self.config.postprocessing_prompt.clone())
                    .build()
                    .unwrap()
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(transcript)
                    .build()
                    .unwrap()
                    .into(),
            ])
            .model(model)
            .build()
            .unwrap();
        let response = self.client.chat().create(request).await.unwrap();
        response.choices.first().unwrap().message.content.clone()
    }

    pub async fn transcribe(&self, audio: Vec<u8>) -> Option<String> {
        let model = self.config.whisper_model.clone();
        let request: CreateTranscriptionRequest = CreateTranscriptionRequestArgs::default()
            .file(AudioInput::from_vec_u8("in.mp3".to_string(), audio))
            .model(model)
            .build()
            .unwrap();
        let response = self.client.audio().transcribe(request).await.unwrap();
        Some(response.text)
    }
}
