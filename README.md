# gpt-auto-test

ChatGPT powered Rust macro that automatically generates test cases for given function.

# Usage

See this example:

```rust
use gpt_auto_test::gpt_auto_test;

#[gpt_auto_test(test_valid, test_div_by_zero)]
fn div_u32(a: u32, b: u32) -> u32 {
    if b == 0 {
        panic!("attempt to divide by zero");
    }
    a / b
}
```

Get ChatGPT API key and set it to your environment variable `OPENAI_API_KEY` before run.

# License

gpt-auto-test is released under the MIT license.
