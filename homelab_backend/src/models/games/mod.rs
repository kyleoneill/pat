pub mod games_db;

use mongodb::bson::{doc, Document};
use rand::seq::SliceRandom;

use super::deserialize_id;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionGameSchema {
    pub connection_categories: [ConnectionCategorySchema; 4],
    pub puzzle_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConnectionCategorySchema {
    pub category_clues: [String; 4],
    pub category_name: String,
}

impl ConnectionCategorySchema {
    pub fn to_doc(&self) -> Document {
        doc! {
            "category_clues": [
                self.category_clues[0].to_owned(),
                self.category_clues[1].to_owned(),
                self.category_clues[2].to_owned(),
                self.category_clues[3].to_owned(),
            ],
            "category_name": self.category_name.to_string()
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConnectionGame {
    #[serde(rename = "_id", deserialize_with = "deserialize_id")]
    pub id: String,
    pub connection_categories: [ConnectionCategory; 4],
    pub puzzle_name: String,
    pub slug: String,
    pub author_id: String,
    pub creation_datetime: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConnectionCategory {
    pub category_clues: [String; 4],
    pub category_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct PlayConnectionGame {
    pub id: String,
    pub author_id: String,
    pub scrambled_clues: [String; 16],
    pub puzzle_name: String,
    pub slug: String,
    pub creation_datetime: i64,
}

impl From<ConnectionGame> for PlayConnectionGame {
    fn from(value: ConnectionGame) -> Self {
        let first = value.connection_categories[0].clone();
        let second = value.connection_categories[1].clone();
        let third = value.connection_categories[2].clone();
        let fourth = value.connection_categories[3].clone();
        let mut scrambled_clues: [String; 16] = first
            .category_clues
            .into_iter()
            .chain(second.category_clues)
            .chain(third.category_clues)
            .chain(fourth.category_clues)
            .collect::<Vec<String>>()
            .try_into()
            .expect("Failed to convert ");
        let mut rng = rand::rng();
        scrambled_clues.shuffle(&mut rng);
        Self {
            id: value.id,
            author_id: value.author_id,
            scrambled_clues,
            puzzle_name: value.puzzle_name,
            slug: value.slug,
            creation_datetime: value.creation_datetime,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MinimalConnectionsGame {
    pub id: String,
    pub puzzle_name: String,
    pub slug: String,
    pub author_id: String,
    pub creation_datetime: i64,
}

impl From<ConnectionGame> for MinimalConnectionsGame {
    fn from(value: ConnectionGame) -> Self {
        Self {
            id: value.id,
            puzzle_name: value.puzzle_name,
            slug: value.slug,
            author_id: value.author_id,
            creation_datetime: value.creation_datetime,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TrySolveRow {
    pub row_name: Option<String>,
    pub correct_guess: bool,
}
