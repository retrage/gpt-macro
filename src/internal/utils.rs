// SPDX-License-Identifier: MIT
// Akira Moroo <retrage01@gmail.com> 2023

use async_openai::types::CreateChatCompletionResponse;

pub fn extract_code(
    response: &CreateChatCompletionResponse,
) -> Result<String, Box<dyn std::error::Error>> {
    let content = response.choices[0]
        .message
        .content
        .clone()
        .expect("No content found.");

    // Remove the code block and remaining explanation text.
    // Extract the test case in the code block. Other parts are removed.
    let code_block = content
        .split("```rust")
        .nth(1)
        .ok_or(format!("No code block start found: {}", content))?
        .split("```")
        .next()
        .ok_or(format!("No code block end found: {}", content))?
        .trim()
        .to_string();

    Ok(code_block)
}
