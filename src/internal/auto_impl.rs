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
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_str, LitStr,
};
use tokio::runtime::Runtime;

use super::utils;

/// Parses the following syntax:
///
/// auto_impl! {
///     $STR_LIT
///     $TOKEN_STREAM
/// }
struct AutoImpl {
    doc: String,
    token_stream: proc_macro2::TokenStream,
}

impl Parse for AutoImpl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let doc = input.parse::<LitStr>()?.value();
        let token_stream = input.parse::<proc_macro2::TokenStream>()?;
        Ok(AutoImpl { doc, token_stream })
    }
}

impl AutoImpl {
    async fn completion(&mut self) -> Result<TokenStream, Box<dyn std::error::Error>> {
        let request = CreateChatCompletionRequestArgs::default()
            .model(utils::MODEL)
            .response_format(utils::structured_output_format())
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are a Rust expert who implements the requested function.")
                    .build()?.into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(format!(
                        "Complete the following Rust source according to the instruction. Return the complete source in the code field. Do not include Markdown code fences in that field.\n\nInstruction: {}\n\nIncomplete Rust source:\n{}",
                        self.doc, self.token_stream
                    ))
                    .build()?.into(),
            ])
            .build()?;

        let client = Client::new();
        let response = client.chat().create(request).await?;

        self.parse_str(&utils::extract_code(&response)?)
    }

    fn parse_str(&self, s: &str) -> Result<TokenStream, Box<dyn std::error::Error>> {
        let expanded = if let Ok(code) = parse_str::<proc_macro2::TokenStream>(s) {
            quote! {
                #code
            }
        } else {
            return Err(format!("Failed to parse the response as Rust code:\n{s}\n").into());
        };

        Ok(TokenStream::from(expanded))
    }
}

pub fn auto_impl_impl(input: TokenStream) -> TokenStream {
    let mut auto_impl = parse_macro_input!(input as AutoImpl);

    let rt = Runtime::new().expect("Failed to create a runtime.");
    rt.block_on(auto_impl.completion())
        .unwrap_or_else(|e| panic!("{}", e))
}
