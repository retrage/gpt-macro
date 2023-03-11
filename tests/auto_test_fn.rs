// SPDX-License-Identifier: MIT
// Akira Moroo <retrage01@gmail.com> 2023

use gpt_macro::auto_test;

#[auto_test(test_valid, test_div_by_zero)]
fn div_u32(a: u32, b: u32) -> u32 {
    if b == 0 {
        panic!("attempt to divide by zero");
    }
    a / b
}

#[auto_test]
fn collaz(n: u32) -> u32 {
    if n % 2 == 0 {
        n / 2
    } else {
        3 * n + 1
    }
}

fn main() {}
