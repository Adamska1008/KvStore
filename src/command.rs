use crate::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    SetCommand { key: String, value: String },
    RemoveCommand { key: String },
}

impl Command {
    pub fn set(key: &str, value: &str) -> Self {
        Command::SetCommand {
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    pub fn rm(key: &str) -> Self {
        Command::RemoveCommand {
            key: key.to_string(),
        }
    }

    pub fn as_json(&self) -> Result<String> {
        let str = serde_json::to_string(&self)?;
        Ok(str)
    }

    pub fn name(&self) -> String {
        match *self {
            Command::SetCommand { .. } => "SetCommand".to_string(),
            Command::RemoveCommand { .. } => "RemoveCommand".to_string(),
        }
    }
}

pub struct CommandPos {
    pub file_stem: u64,
    pub offset: u64,
    pub len: u64,
}

impl CommandPos {
    pub fn new(file_stem: u64, offset: u64, len: u64) -> Self {
        Self {
            file_stem,
            offset,
            len,
        }
    }
}
