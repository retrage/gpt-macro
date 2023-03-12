// SPDX-License-Identifier: MIT
// Akira Moroo <retrage01@gmail.com> 2023

// Ask GPT-3.5 to complete the given function.
// Use hyper to send a POST request to the GPT-3.5 API.

use hyper::header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use hyper::{Body, Client, Request, Uri};
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::internal::completion::CodeCompletion;

#[derive(Deserialize, Serialize, Debug)]
struct CompletionRequest {
    model: String,
    prompt: String,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Deserialize, Serialize)]
struct CompletionResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<CompletionChoice>,
    usage: CompletionUsage,
}

#[derive(Debug, Deserialize, Serialize)]
struct CompletionChoice {
    text: String,
    index: u32,
    logprobs: Option<u32>,
    finish_reason: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct CompletionUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

pub struct TextCompletion {
    request: CompletionRequest,
    response: Option<CompletionResponse>,
}

impl TextCompletion {
    const URL: &'static str = "https://api.openai.com/v1/completions";
    const MODEL: &'static str = "text-davinci-003";

    fn add_prompt(&mut self, content: String) {
        self.request.prompt.push('\n');
        self.request.prompt.push_str(&content);
    }

    async fn completion(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY is not set");
        let uri: Uri = Self::URL.parse()?;

        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);

        let body = Body::from(serde_json::to_string(&self.request)?);

        let mut request = Request::new(body);

        *request.method_mut() = hyper::Method::POST;
        *request.uri_mut() = uri.clone();

        request
            .headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
        );

        let response = client.request(request).await?;
        let body_bytes = hyper::body::to_bytes(response.into_body()).await?;
        let body_str = String::from_utf8(body_bytes.to_vec())?;

        let response: CompletionResponse = serde_json::from_str(&body_str)?;

        let content = response.choices[0].text.clone();
        println!("Response from {}:\n{}", self.request.model, content);

        self.response = Some(response);

        Ok(())
    }

    fn extract_code(&self) -> Result<String, Box<dyn std::error::Error>> {
        let content = self.response.as_ref().ok_or("No response")?.choices[0]
            .text
            .clone();
        // Remove the code block and remaining explanation text.
        // Extract the test case in the code block. Other parts are removed.
        let code_block = content
            .split("```rust")
            .nth(1)
            .ok_or(format!("No code block start found: {}", content))?
            .split("```")
            .next()
            .ok_or(format!("No code block end found: {}", content))?
            .trim()
            .to_string();

        Ok(code_block)
    }
}

impl CodeCompletion for TextCompletion {
    fn new() -> Self {
        Self {
            request: CompletionRequest {
                model: Self::MODEL.to_string(),
                prompt: String::new(),
                max_tokens: 1024,
                temperature: 0.0,
            },
            response: None,
        }
    }

    fn init(&mut self, init_prompt: String) {
        self.add_prompt(init_prompt);
    }

    fn add_context(&mut self, context: String) {
        self.add_prompt(context)
    }

    fn code_completion(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let rt = Runtime::new()?;

        rt.block_on(self.completion())?;

        self.extract_code()
    }
}
