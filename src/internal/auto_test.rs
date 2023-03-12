// SPDX-License-Identifier: MIT
// Akira Moroo <retrage01@gmail.com> 2023

use proc_macro::TokenStream;
use std::collections::HashSet;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Ident, Token,
};

use crate::internal::chatgpt;

/// Parses a list of test function names separated by commas.
///
/// test_valid, test_div_by_zero
///
/// The function name is used to generate the test function name.
struct Args {
    test_names: HashSet<Ident>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let test_names = input.parse_terminated::<Ident, Token![,]>(Ident::parse)?;
        Ok(Args {
            test_names: test_names.into_iter().collect(),
        })
    }
}

pub fn auto_test_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the list of test function names that should be generated.
    let args = parse_macro_input!(args as Args);

    chatgpt::AutoTest::<chatgpt::ChatGPT>::new(input.into())
        .completion(args.test_names)
        .unwrap_or_else(|e| panic!("{}", e))
        .into()
}
