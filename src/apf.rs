use std::{fs, path::Path};

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

pub fn parse_json(json_path: &str) -> Vec<DataSlot> {
    let json = fs::read_to_string(&json_path).expect("Could not find data slot JSON file");
    let json_directory = Path::new(&json_path)
        .canonicalize()
        .expect("Could not resolve data slot JSON file");
    let json_directory = json_directory
        .parent()
        .expect("Could not find directory containing data slot JSON file");

    let mut data =
        serde_json::from_str::<DataJson>(&json).expect("Could not parse data slot JSON file");

    data.data.data_slots.iter_mut().for_each(|slot| {
        let path = Path::new(&slot.filename);
        let resolved_path = json_directory
            .join(path)
            .canonicalize()
            // If we cannot resolve this path, just stick the original filepath back in
            .unwrap_or_else(|_| path.to_path_buf())
            .into_os_string()
            .into_string()
            .unwrap();
        slot.filename = resolved_path;
    });

    data.data.data_slots
}
