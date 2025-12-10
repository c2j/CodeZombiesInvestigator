//! Python symbol extractor

use crate::Result;

pub struct PythonExtractor;

impl PythonExtractor {
    pub fn new() -> Self { Self }
    pub fn extract_symbols(&self, code: &str) -> Result<()> {
        Ok(())
    }
}