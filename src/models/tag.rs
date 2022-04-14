use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    guild_id: String,
    author_id: String,
    command_id: Option<String>,

    name: String,

    tagscript: String,
    options: Vec<TagOption>,
    uses: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TagOption {
    name: String,
    description: String,
    required: bool,
    #[serde(rename = "type")]
    ty: u8,
}
