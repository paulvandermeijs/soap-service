//! XML serialization helpers

use serde::{Deserialize, Serialize};

pub fn serialize_to_xml<T: Serialize>(data: &T) -> Result<String, String> {
    quick_xml::se::to_string(data).map_err(|e| e.to_string())
}

pub fn deserialize_from_xml<T: for<'de> Deserialize<'de>>(xml: &str) -> Result<T, String> {
    quick_xml::de::from_str(xml).map_err(|e| e.to_string())
}