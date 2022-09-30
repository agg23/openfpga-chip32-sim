use serde::Deserialize;

#[derive(Deserialize)]
pub struct DataJson {
    pub data: DataJsonData,
}

#[derive(Deserialize)]
pub struct DataJsonData {
    // magic: String,
    pub data_slots: Vec<DataSlot>,
}

#[derive(Deserialize)]
pub struct DataSlot {
    // TODO: This can also be a string of hex
    pub id: u32,
    pub filename: String,
    // We don't care about the other fields
}
