// SPDX-License-Identifier: MIT
// Akira Moroo <retrage01@gmail.com> 2023

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_str, LitStr,
};

use crate::internal::{chatgpt::ChatGPT, completion::CodeCompletion};

/// Parses the following syntax:
///
/// ```
/// auto_impl! {
///     $STR_LIT
///     $TOKEN_STREAM
/// }
/// ```
struct AutoImpl<C: CodeCompletion> {
    doc: String,
    token_stream: proc_macro2::TokenStream,
    code_completion: C,
}

impl<C: CodeCompletion> Parse for AutoImpl<C> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let doc = input.parse::<LitStr>()?.value();
        let token_stream = input.parse::<proc_macro2::TokenStream>()?;
        Ok(AutoImpl {
            doc,
            token_stream,
            code_completion: C::new(),
        })
    }
}

impl<C: CodeCompletion> AutoImpl<C> {
    fn completion(&mut self) -> Result<TokenStream, Box<dyn std::error::Error>> {
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

pub fn auto_impl_impl(input: TokenStream) -> TokenStream {
    let mut auto_impl = parse_macro_input!(input as AutoImpl<ChatGPT>);

    auto_impl.completion().unwrap_or_else(|e| panic!("{}", e))
}
