use serde::Deserialize;

use crate::util::serde::serde_string_or_int;

#[derive(Deserialize)]
pub struct DataJson {
    pub data: DataJsonData,
}

#[derive(Deserialize)]
pub struct DataJsonData {
    // magic: String,
    pub data_slots: Vec<DataSlot>,
}

#[derive(Clone, Deserialize)]
pub struct DataSlot {
    // TODO: This can also be a string of hex
    #[serde(deserialize_with = "serde_string_or_int")]
    pub id: u32,
    pub filename: String,
    // We don't care about the other fields
}
