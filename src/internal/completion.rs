// SPDX-License-Identifier: MIT
// Akira Moroo <retrage01@gmail.com> 2023

pub trait CodeCompletion {
    fn new() -> Self;
    fn init(&mut self, init_prompt: String);
    fn add_context(&mut self, context: String);
    fn code_completion(&mut self) -> Result<String, Box<dyn std::error::Error>>;
}
