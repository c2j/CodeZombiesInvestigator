//! JavaScript symbol extractor

use crate::Result;

pub struct JavaScriptExtractor;

impl JavaScriptExtractor {
    pub fn new() -> Self { Self }
    pub fn extract_symbols(&self, code: &str) -> Result<()> {
        Ok(())
    }
}