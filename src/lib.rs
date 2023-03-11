// SPDX-License-Identifier: MIT
// Akira Moroo <retrage01@gmail.com> 2023

use proc_macro::TokenStream;
use std::collections::HashSet;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Ident, Token,
};

mod chatgpt;

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

/// Attribute macro for automatically generating tests for functions.
///
/// # Example
///
/// ```
/// use r#gpt_auto_test::auto_test;
///
/// #[auto_test(test_valid, test_div_by_zero)]
/// fn div_u32(a: u32, b: u32) -> u32 {
///    a / b
/// }
///
/// #[auto_test]
/// fn collaz(n: u32) -> u32 {
///     if n % 2 == 0 {
///         n / 2
///     } else {
///         3 * n + 1
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn auto_test(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the list of test function names that should be generated.
    let args = parse_macro_input!(args as Args);

    match chatgpt::generate_tests(input, args.test_names) {
        Ok(output) => output,
        Err(e) => panic!("{}", e),
    }
}
