#![no_std]

pub struct Loot {
    pub bytes: [u8; 16],
}

pub enum LootError {
    Empty,
}

pub type LootResult = Result<Option<Loot>, LootError>;
