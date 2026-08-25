// SPDX-License-Identifier: MIT
// Akira Moroo <retrage01@gmail.com> 2023

use async_openai::types::chat::{
    CreateChatCompletionResponse, ResponseFormat, ResponseFormatJsonSchema,
};
use serde_json::Value;

pub const MODEL: &str = "openai/gpt-5.6-luna";

/// Build the strict JSON schema used by both macros.
pub fn structured_output_format() -> ResponseFormat {
    ResponseFormat::JsonSchema {
        json_schema: ResponseFormatJsonSchema {
            description: Some("Generated Rust source code.".to_owned()),
            name: "rust_code".to_owned(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string"
                    }
                },
                "required": ["code"],
                "additionalProperties": false
            }),
            strict: Some(true),
        },
    }
}

/// Extract the generated Rust source from a structured chat completion.
pub fn extract_code(
    response: &CreateChatCompletionResponse,
) -> Result<String, Box<dyn std::error::Error>> {
    let content = response
        .choices
        .first()
        .ok_or_else(|| invalid_response("No choices found in the response."))?
        .message
        .content
        .as_deref()
        .ok_or_else(|| invalid_response("No content found in the response."))?;

    extract_code_from_content(content)
}

fn extract_code_from_content(content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let value: Value = serde_json::from_str(content).map_err(|error| {
        invalid_response(format!(
            "Failed to parse the structured output as JSON: {error}\n{content}"
        ))
    })?;

    let code = value
        .get("code")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid_response("Structured output is missing the code string."))?;

    if code.trim().is_empty() {
        return Err(invalid_response(
            "Structured output contains an empty code string.",
        ));
    }

    Ok(code.trim().to_owned())
}

fn invalid_response(message: impl Into<String>) -> Box<dyn std::error::Error> {
    Box::new(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        message.into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_code_from_structured_output() {
        let code = extract_code_from_content(r#"{"code":"fn answer() -> u32 { 42 }"}"#)
            .expect("structured output should contain code");

        assert_eq!(code, "fn answer() -> u32 { 42 }");
    }

    #[test]
    fn rejects_output_without_code() {
        let error = extract_code_from_content(r#"{"explanation":"not code"}"#)
            .expect_err("missing code should be rejected");

        assert!(error.to_string().contains("missing the code string"));
    }

    #[test]
    fn creates_a_strict_code_schema() {
        let ResponseFormat::JsonSchema { json_schema } = structured_output_format() else {
            panic!("expected a JSON schema response format");
        };

        assert_eq!(json_schema.name, "rust_code");
        assert_eq!(json_schema.strict, Some(true));
        assert_eq!(json_schema.schema["properties"]["code"]["type"], "string");
    }
}
