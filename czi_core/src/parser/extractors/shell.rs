//! Shell script symbol extractor

use crate::Result;

pub struct ShellExtractor;

impl ShellExtractor {
    pub fn new() -> Self { Self }
    pub fn extract_symbols(&self, code: &str) -> Result<()> {
        Ok(())
    }
}