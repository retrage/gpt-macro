// SPDX-License-Identifier: MIT
// Akira Moroo <retrage01@gmail.com> 2023

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::collections::HashSet;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_str, Ident, ItemFn, ItemMod, Token,
};

use crate::internal::{chatgpt::ChatGPT, completion::CodeCompletion};

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
        let test_names = input.parse_terminated::<Ident, Token![,]>(Ident::parse)?;
        Ok(Args {
            test_names: test_names.into_iter().collect(),
        })
    }
}

struct AutoTest<C: CodeCompletion> {
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

    pub fn completion(&mut self, args: Args) -> Result<TokenStream, Box<dyn std::error::Error>> {
        let mut output = self.token_stream.clone();

        let init_prompt =
            "You are a Rust expert who can generate perfect tests for the given function.";
        self.code_completion.init(init_prompt.to_string());
        self.code_completion.add_context(format!(
            "Read this Rust function:\n```rust\n{}\n```",
            self.token_stream,
        ));

        if args.test_names.is_empty() {
            self.do_completion(None)?.to_tokens(&mut output);
        } else {
            for test_name in args.test_names {
                self.do_completion(Some(test_name))?.to_tokens(&mut output);
            }
        }

        Ok(TokenStream::from(output))
    }

    fn parse_str(&self, s: &str) -> Result<proc_macro2::TokenStream, Box<dyn std::error::Error>> {
        let expanded = if let Ok(test_case) = parse_str::<ItemFn>(s) {
            quote! {
                #test_case
            }
        } else if let Ok(test_case) = parse_str::<ItemMod>(s) {
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

pub fn auto_test_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the list of test function names that should be generated.
    let args = parse_macro_input!(args as Args);

    AutoTest::<ChatGPT>::new(input.into())
        .completion(args)
        .unwrap_or_else(|e| panic!("{}", e))
}
