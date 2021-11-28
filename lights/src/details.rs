use serde::{Deserialize, Serialize};

use crate::effects::EffectType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Details {
    pub length: usize,
    pub effect: EffectType,
    pub brightness: u8,
}

impl Default for Details {
    fn default() -> Self {
        Self {
            length: 100,
            brightness: 150,
            effect: Default::default(),
        }
    }
}
