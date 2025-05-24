//! Convert bot-specific export formats into the Proxy Bot Unified Export Format

use serde_json::Value;

mod error;
pub use self::error::*;

pub mod pluralkit;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ExportFormat {
    PluralKit(Box<pluralkit::PKExport>),
}

impl ExportFormat {
    pub fn detect(value: &Value) -> Result<Self, DetectionError> {
        if value
            .get("version")
            .and_then(|v| v.as_u64())
            .is_some_and(|ver| ver == 2)
            && value
                .get("id")
                .and_then(|v| v.as_str())
                .is_some_and(|id| id.len() == 5 || id.len() == 6)
            && value.get("switches").is_some_and(|v| v.is_array())
        {
            return Ok(Self::PluralKit(Box::new(serde_json::from_value(
                value.clone(),
            )?)));
        }

        Err(DetectionError::UnknownFormat)
    }

    pub fn try_convert(&self) -> Result<proxybot_export::UnifiedExport, ConversionError> {
        match self {
            Self::PluralKit(pk) => Ok(pk.try_convert()?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_pluralkit_export() {
        let export: Value =
            serde_json::from_str(include_str!("../testdata/pluralkit-exmpl.json")).unwrap();

        assert!(matches!(
            ExportFormat::detect(&export).unwrap(),
            ExportFormat::PluralKit(_),
        ));
    }
}
