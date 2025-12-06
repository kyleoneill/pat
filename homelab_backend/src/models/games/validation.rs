use mongodb::bson::{doc, Document};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateConnectionGameSchema {
    pub connection_categories: [CreateConnectionCategorySchema; 4],
    pub puzzle_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateConnectionCategorySchema {
    pub category_clues: [String; 4],
    pub category_name: String,
}

impl CreateConnectionCategorySchema {
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
