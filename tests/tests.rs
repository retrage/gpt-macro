// SPDX-License-Identifier: MIT
// Akira Moroo <retrage01@gmail.com> 2023

#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/auto_test_fn.rs");
    t.pass("tests/auto_impl_fn.rs");
}
