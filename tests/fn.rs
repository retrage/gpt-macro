// SPDX-License-Identifier: MIT
// Akira Moroo <retrage01@gmail.com> 2023

use gpt_auto_test::gpt_auto_test;

#[gpt_auto_test(test_valid, test_div_by_zero)]
fn div_u32(a: u32, b: u32) -> u32 {
    if b == 0 {
        panic!("attempt to divide by zero");
    }
    a / b
}

fn main() {}
