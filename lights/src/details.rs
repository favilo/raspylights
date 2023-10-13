use serde::{Deserialize, Serialize};

use crate::effects::EffectType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Details {
    pub effect: EffectType,
    pub length: usize,
    pub brightness: u8,
    pub name: String,
}

impl Default for Details {
    fn default() -> Self {
        Self {
            length: 100,
            brightness: 150,
            effect: Default::default(),
            name: Default::default(),
        }
    }
}
