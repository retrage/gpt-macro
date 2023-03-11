// SPDX-License-Identifier: MIT
// Akira Moroo <retrage01@gmail.com> 2023

use proc_macro::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, LitStr,
};

use crate::internal::chatgpt;

/// Parses the following syntax:
///
/// ```
/// auto_impl! {
///     $STR_LIT
///     $TOKEN_STREAM
/// }
/// ```
struct AutoImpl {
    doc: String,
    token: proc_macro2::TokenStream,
}

impl Parse for AutoImpl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let doc = input.parse::<LitStr>()?.value();
        let token = input.parse::<proc_macro2::TokenStream>()?;
        Ok(AutoImpl { doc, token })
    }
}

pub fn auto_impl_impl(input: TokenStream) -> TokenStream {
    let AutoImpl { doc, token } = parse_macro_input!(input as AutoImpl);
    match chatgpt::generate_impl(doc, token) {
        Ok(output) => output,
        Err(e) => panic!("{}", e),
    }
}
