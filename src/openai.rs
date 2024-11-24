/// Use OpenAI to postprocess a transcript.

use crate::config;

use async_openai::{
    types::CreateChatCompletionRequestArgs,
    types::CreateChatCompletionRequest,
    types::ChatCompletionRequestSystemMessageArgs,
    types::ChatCompletionRequestUserMessageArgs,
    Client,
    config::OpenAIConfig as LibOpenAIConfig
};

pub async fn postprocess(transcript: &str, prompt: &str, openai_config: &config::OpenaiConfig) -> Option<String> {
    let api_key = openai_config.api_key.clone();
    let model = openai_config.postprocessing_model.clone();
    let client_config = LibOpenAIConfig::new().with_api_key(api_key);
    let client = Client::with_config(client_config);

    let request: CreateChatCompletionRequest = CreateChatCompletionRequestArgs::default()
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(prompt)
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
    let response = client.chat().create(request).await.unwrap();
    response.choices.first().unwrap().message.content.clone()
}
