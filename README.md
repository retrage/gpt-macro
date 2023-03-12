# gpt-macro

ChatGPT powered Rust proc macro that generates code at compile-time.

## Implemented Macros

* `auto_impl!{}`
* `#[auto_test(...)]`

## Usage

Get ChatGPT API key and set it to your environment variable `OPENAI_API_KEY` before run.

### `auto_impl!{}`

Syntax:

```rust
auto_impl! {
    $STR_LIT
    $TOKEN_STREAM
}
```

where `$STR_LIT` is a prompt string literal, and `$TOKEN_STREAM` is target code.

Example:

```rust
use gpt_macro::auto_impl;

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
```

As you can see, the `fizzbuzz()` implementation is incomplete, so the build fails without `auto_impl!{}`. The macro parses given prompt and target code, and asks ChatGPT to fill the code when expanding the macro. It replaces the target with code extracted from ChatGPT response. Then Rust compiler continues compiling the code.

Response Example:

```rust
fn fizzbuzz(n: u32) -> String {
    if n % 3 == 0 && n % 5 == 0 {
        return String::from("fizzbuzz");
    } else if n % 3 == 0 {
        return String::from("fizz");
    } else if n % 5 == 0 {
        return String::from("buzz");
    } else {
        return n.to_string();
    }
}

#[test]
fn test_fizzbuzz() {
    assert_eq!(fizzbuzz(3), "fizz");
    assert_eq!(fizzbuzz(5), "buzz");
    assert_eq!(fizzbuzz(15), "fizzbuzz");
    assert_eq!(fizzbuzz(1), "1");
}
```

### `#[auto_test]`

See this example:

```rust
use gpt_macro::auto_test;

#[auto_test(test_valid, test_div_by_zero)]
fn div_u32(a: u32, b: u32) -> u32 {
    if b == 0 {
        panic!("attempt to divide by zero");
    }
    a / b
}
```

## Supported Models

* ChatGPT: `gpt-3.5-turbo` (default)
* Text Completion: `text-davinci-003` (Specify `davinci` feature to enable it)

## License

gpt-macro is released under the MIT license.
