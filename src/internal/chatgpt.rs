// SPDX-License-Identifier: MIT
// Akira Moroo <retrage01@gmail.com> 2023

// Ask ChatGPT to generate cases for the given function.
// Use hyper to send a POST request to the ChatGPT API.

use std::collections::HashSet;

use hyper::header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use hyper::{Body, Client, Request, Uri};
use hyper_tls::HttpsConnector;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use serde::{Deserialize, Serialize};
use syn::{parse_str, Ident, ItemFn, ItemMod};
use tokio::runtime::Runtime;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum Role {
    User,
    System,
    Assistant,
}

#[derive(Deserialize, Serialize, Debug)]
struct ChatMessage {
    role: Role,
    content: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Chat {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ChatCompletion {
    id: String,
    object: String,
    created: u64,
    choices: Vec<ChatChoice>,
    usage: ChatUsage,
}

#[derive(Debug, Deserialize, Serialize)]
struct ChatChoice {
    index: u32,
    message: ChatMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ChatUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

pub struct ChatGPT {
    chat: Chat,
}

impl ChatGPT {
    const URL: &'static str = "https://api.openai.com/v1/chat/completions";
    const MODEL: &'static str = "gpt-3.5-turbo";

    fn add_message(&mut self, role: Role, content: String) {
        self.chat.messages.push(ChatMessage { role, content });
    }

    async fn completion(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY is not set");
        let uri: Uri = Self::URL.parse()?;

        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);

        let body = Body::from(serde_json::to_string(&mut self.chat)?);

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

        let chat_completion: ChatCompletion = serde_json::from_str(&body_str)?;

        let content = chat_completion.choices[0].message.content.clone();

        println!("Response from ChatGPT:\n{}", content);

        self.add_message(Role::Assistant, content);

        Ok(())
    }

    fn extract_code(&self) -> Result<String, Box<dyn std::error::Error>> {
        let last_content = self.chat.messages[self.chat.messages.len() - 1]
            .content
            .clone();
        // Remove the code block and remaining explanation text.
        // Extract the test case in the code block. Other parts are removed.
        let code_block = last_content
            .split("```rust")
            .nth(1)
            .ok_or(format!("No code block start found: {}", last_content))?
            .split("```")
            .next()
            .ok_or(format!("No code block end found: {}", last_content))?
            .trim()
            .to_string();

        Ok(code_block)
    }
}

pub struct AutoImpl<C: CodeCompletion> {
    token_stream: proc_macro2::TokenStream,
    doc: String,
    code_completion: C,
}

impl<C: CodeCompletion> AutoImpl<C> {
    pub fn new(token_stream: proc_macro2::TokenStream, doc: String) -> Self {
        Self {
            token_stream,
            doc,
            code_completion: C::new(),
        }
    }

    pub fn completion(&mut self) -> Result<TokenStream, Box<dyn std::error::Error>> {
        let init_prompt = "You are a Rust expert who can implement the given function.";
        self.code_completion.init(init_prompt.to_string());
        self.code_completion.add_context(format!(
            "Read this incomplete Rust code:\n```rust\n{}\n```",
            self.token_stream
        ));
        self.code_completion.add_context(format!(
            "Complete the Rust code that follows this instruction: '{}'. Your response must start with code block '```rust'.",
            self.doc
        ));

        let code_text = self.code_completion.code_completion()?;

        self.parse_str(&code_text)
    }

    fn parse_str(&self, s: &str) -> Result<TokenStream, Box<dyn std::error::Error>> {
        let expanded = if let Ok(code) = parse_str::<proc_macro2::TokenStream>(s) {
            quote! {
                #code
            }
        } else {
            return Err(format!("Failed to parse the response as Rust code:\n{}\n", s).into());
        };

        Ok(TokenStream::from(expanded))
    }
}

pub struct AutoTest<C: CodeCompletion> {
    token_stream: proc_macro2::TokenStream,
    code_completion: C,
}

impl<C: CodeCompletion> AutoTest<C> {
    pub fn new(token_stream: proc_macro2::TokenStream) -> Self {
        Self {
            token_stream,
            code_completion: C::new(),
        }
    }

    fn do_completion(
        &mut self,
        test_name: Option<Ident>,
    ) -> Result<proc_macro2::TokenStream, Box<dyn std::error::Error>> {
        let prompt = if let Some(test_name) = test_name {
            format!(
                "Write a test case `{}` for the function in Markdown code snippet style. Your response must start with code block '```rust'.",
                test_name
            )
        } else {
            "Write a test case for the function as much as possible in Markdown code snippet style. Your response must start with code block '```rust'.".to_string()
        };
        self.code_completion.add_context(prompt);

        let test_text = self.code_completion.code_completion()?;

        self.parse_str(&test_text)
    }

    pub fn completion(
        &mut self,
        test_names: HashSet<Ident>,
    ) -> Result<TokenStream, Box<dyn std::error::Error>> {
        let mut output = self.token_stream.clone();

        let init_prompt =
            "You are a Rust expert who can generate perfect tests for the given function.";
        self.code_completion.init(init_prompt.to_string());
        self.code_completion.add_context(format!(
            "Read this Rust function:\n```rust\n{}\n```",
            self.token_stream,
        ));

        if test_names.is_empty() {
            self.do_completion(None)?.to_tokens(&mut output);
        } else {
            for test_name in test_names {
                self.do_completion(Some(test_name))?.to_tokens(&mut output);
            }
        }

        Ok(TokenStream::from(output))
    }

    fn parse_str(&self, s: &str) -> Result<proc_macro2::TokenStream, Box<dyn std::error::Error>> {
        let expanded = if let Ok(test_case) = parse_str::<ItemFn>(&s) {
            quote! {
                #test_case
            }
        } else if let Ok(test_case) = parse_str::<ItemMod>(&s) {
            quote! {
                #test_case
            }
        } else {
            return Err(format!(
                "Failed to parse the test case as a function or a module:\n{}\n",
                s
            )
            .into());
        };

        Ok(expanded)
    }
}

pub trait CodeCompletion {
    fn new() -> Self;
    fn init(&mut self, init_prompt: String);
    fn add_context(&mut self, context: String);
    fn code_completion(&mut self) -> Result<String, Box<dyn std::error::Error>>;
}

impl CodeCompletion for ChatGPT {
    fn new() -> Self {
        Self {
            chat: Chat {
                model: Self::MODEL.to_string(),
                messages: vec![],
            },
        }
    }

    fn init(&mut self, init_prompt: String) {
        self.add_message(Role::System, init_prompt);
    }

    fn add_context(&mut self, context: String) {
        self.add_message(Role::User, context);
    }

    fn code_completion(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let rt = Runtime::new()?;

        rt.block_on(self.completion())?;

        Ok(self.extract_code()?)
    }
}
