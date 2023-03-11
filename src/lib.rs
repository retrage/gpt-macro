// SPDX-License-Identifier: MIT
// Akira Moroo <retrage01@gmail.com> 2023

use internal::auto_test::auto_test_impl;
use proc_macro::TokenStream;

mod internal;

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
    auto_test_impl(args, input)
}
