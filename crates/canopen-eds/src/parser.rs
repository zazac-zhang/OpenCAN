//! EDS file parser.

use crate::model::EdsFile;

pub struct EdsParser;

impl EdsParser {
    pub fn from_str(content: &str) -> Result<EdsFile, String> {
        // TODO: Implement EDS parsing
        Err("EDS parsing not yet implemented".to_string())
    }
}
