use csv::Reader;
use serde::{Deserialize, Serialize};

pub struct Shop(Vec<Item>);

#[derive(Serialize, Deserialize)]
struct Item {
    name: String,
    cost: u64,
}

impl Shop {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from_file(shop_file: &str) -> Result<(), csv::Error> {
        let mut shop = Shop::new();
        let mut rdr = Reader::from_path(shop_file)?;

        for record in rdr.deserialize() {
            shop.0.push(record?);
        }

        Ok(())
    }
}
