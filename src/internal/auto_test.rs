// SPDX-License-Identifier: MIT
// Akira Moroo <retrage01@gmail.com> 2023

use async_openai::{
    types::chat::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::collections::HashSet;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_str, Ident, Token,
};
use tokio::runtime::Runtime;

use super::utils;

/// Parses a list of test function names separated by commas.
///
/// test_valid, test_div_by_zero
///
/// The function name is used to generate the test function name.
struct Args {
    test_names: HashSet<Ident>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let test_names = input.parse_terminated(Ident::parse, Token![,])?;
        Ok(Args {
            test_names: test_names.into_iter().collect(),
        })
    }
}

struct AutoTest {
    token_stream: proc_macro2::TokenStream,
}

impl AutoTest {
    fn new(token_stream: proc_macro2::TokenStream) -> Self {
        Self { token_stream }
    }

    async fn completion(&mut self, args: Args) -> Result<TokenStream, Box<dyn std::error::Error>> {
        let mut output = self.token_stream.clone();

        let mut messages =
            vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(
                    "You are a Rust expert who can generate perfect tests for the given function.",
                )
                .build()?.into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(format!(
                    "Read this Rust function:\n```rust\n{}\n```",
                    self.token_stream
                ))
                .build()?.into(),
        ];

        if args.test_names.is_empty() {
            messages.push(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(
                        "Write a test case for the function as much as possible in Markdown code snippet style. Your response must start with code block '```rust'.",
                    )
                    .build()?.into(),
            );
        } else {
            for test_name in args.test_names {
                messages.push(
                    ChatCompletionRequestUserMessageArgs::default()
                        .content(
                            format!(
                                "Write a test case `{test_name}` for the function in Markdown code snippet style. Your response must start with code block '```rust'."
                            )
                        )
                        .build()?.into(),
                );
            }
        }

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo")
            .messages(messages)
            .build()?;

        let client = Client::new();
        let response = client.chat().create(request).await?;

        let test_case = self.parse_str(&utils::extract_code(&response)?)?;
        test_case.to_tokens(&mut output);

        Ok(TokenStream::from(output))
    }

    fn parse_str(&self, s: &str) -> Result<proc_macro2::TokenStream, Box<dyn std::error::Error>> {
        let expanded = if let Ok(test_case) = parse_str::<proc_macro2::TokenStream>(s) {
            quote! {
                #test_case
            }
        } else {
            return Err(format!("Failed to parse the response as Rust code:\n{s}\n").into());
        };

        Ok(expanded)
    }
}

pub fn auto_test_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the list of test function names that should be generated.
    let args = parse_macro_input!(args as Args);

    let mut auto_test = AutoTest::new(input.into());

    let rt = Runtime::new().expect("Failed to create a runtime.");
    rt.block_on(auto_test.completion(args))
        .unwrap_or_else(|e| panic!("{}", e))
}
