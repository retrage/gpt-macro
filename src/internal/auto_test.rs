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
use std::{collections::HashSet, fmt::Write};
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

        let mut instructions = String::from(
            "Generate Rust test function(s) for the function below. Return only the generated test source in the code field. Do not include Markdown code fences in that field.",
        );
        if args.test_names.is_empty() {
            instructions.push_str(" Choose appropriate test function names.");
        } else {
            instructions.push_str(" Generate exactly these test function names: ");
            let mut test_names = args
                .test_names
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            test_names.sort();
            let _ = write!(instructions, "{}", test_names.join(", "));
        }

        let request = CreateChatCompletionRequestArgs::default()
            .model(utils::MODEL)
            .response_format(utils::structured_output_format())
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are a Rust expert who writes useful, compiling tests.")
                    .build()?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(format!(
                        "{}\n\nFunction source:\n{}",
                        instructions, self.token_stream
                    ))
                    .build()?
                    .into(),
            ])
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
