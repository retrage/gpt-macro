// SPDX-License-Identifier: MIT
// Akira Moroo <retrage01@gmail.com> 2023

use gpt_auto_test::auto_impl;

auto_impl! {
    "Return fizz if the number is divisible by 3, buzz if the number is divisible by 5, and fizzbuzz if the number is divisible by both 3 and 5."
    fn fizzbuzz(n: u32) -> String {
    }

    #[test]
    fn test_fizzbuzz() {
        assert_eq!(fizzbuzz(3), "fizz");
        assert_eq!(fizzbuzz(5), "buzz");
        assert_eq!(fizzbuzz(15), "fizzbuzz");
        assert_eq!(fizzbuzz(1), "1");
    }
}

fn main() {}
