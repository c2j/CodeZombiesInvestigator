//! Java symbol extractor

use crate::Result;

pub struct JavaExtractor;

impl JavaExtractor {
    pub fn new() -> Self { Self }
    pub fn extract_symbols(&self, code: &str) -> Result<()> {
        Ok(())
    }
}