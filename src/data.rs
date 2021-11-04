use chrono::{DateTime, Utc};
use std::fs;
use anyhow::Result;
use rand::Rng;
use rand::distributions::Alphanumeric;
use tui::widgets::ListState;
use serde::{Deserialize, Serialize};

const DB_PATH: &str = "./data/db.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct Pet {
    pub id: usize,
    pub name: String,
    pub category: String,
    pub age: usize,
    pub created_at: DateTime<Utc>,
}

pub fn read_db() -> Result<Vec<Pet>> {
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Vec<Pet> = serde_json::from_str(&db_content)?;
    Ok(parsed.to_vec())
}

pub fn add_random_pet_to_db() -> Result<Vec<Pet>> {
    let mut rng = rand::thread_rng();
    let mut data = read_db()?;
    let catsdogs = match rng.gen_range(0, 1) {
        0 => "cats",
        _ => "dogs",
    };
    let random_pet = Pet {
        id: rng.gen_range(0, 999999),
        name: rng.sample_iter(Alphanumeric).take(10).collect(),
        category: catsdogs.to_owned(),
        age: rng.gen_range(1, 15),
        created_at: Utc::now(),
    };

    data.push(random_pet);
    fs::write(DB_PATH, &serde_json::to_vec(&data)?)?;
    Ok(data)
}

pub fn remove_pet_at_index(pet_list_state: &mut ListState) -> Result<()> {
    if let Some(selected) = pet_list_state.selected() {
        let mut data = read_db()?;
        data.remove(selected);
        fs::write(DB_PATH, &serde_json::to_vec(&data)?)?;
        // 削除したら一つ前の要素を選択させる
        pet_list_state.select(Some(selected - 1));
    }
    Ok(())
}