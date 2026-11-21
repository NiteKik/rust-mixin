#![no_std]

#[mixin_sdk::targets("demo::loot")]
pub mod loot {
    fn base_count(level: i32) -> i32 {
        level + 1
    }

    pub fn drop_count(level: i32, lucky: bool, multiplier: f32, cap: u64) -> i32 {
        let count = base_count(level);
        let scaled = (count as f32 * multiplier) as i32 + i32::from(lucky);
        (scaled.max(0) as u64).min(cap) as i32
    }

    pub fn gate(open: bool) -> bool {
        open
    }

    pub fn narrow(value: i8) -> i8 {
        value
    }

    pub fn float(value: f32) -> f32 {
        value
    }

    pub fn adjust(value: f64, offset: i32) -> f64 {
        value + f64::from(offset)
    }

    pub fn max_cap() -> u64 {
        u64::MAX
    }

    pub fn unsigned_count(level: i32) -> u32 {
        base_count(level) as u32
    }

    pub fn signed16(value: i16) -> i16 {
        value
    }

    pub fn signed64(value: i64) -> i64 {
        value
    }

    pub fn signed_size(value: isize) -> isize {
        value
    }

    pub fn unsigned8(value: u8) -> u8 {
        value
    }

    pub fn unsigned16(value: u16) -> u16 {
        value
    }

    pub fn unsigned_size(value: usize) -> usize {
        value
    }
}

#[mixin_sdk::targets("demo::returns")]
pub mod returns {
    use example_return_types::{Loot, LootError, LootResult};

    pub fn bytes(seed: u8) -> [u8; 16] {
        if seed == 0 {
            return [1; 16];
        }
        [seed; 16]
    }

    pub fn vector(force: f32) -> (f32, f32) {
        (force, -1.0)
    }

    pub fn loot(seed: u8) -> LootResult {
        let seed = if seed == 0 {
            Err(LootError::Empty)
        } else {
            Ok(seed)
        }?;
        Ok(Some(Loot { bytes: [seed; 16] }))
    }

    pub fn loot_first_byte(seed: u8) -> u8 {
        match loot(seed) {
            Ok(Some(loot)) => loot.bytes[0],
            Ok(None) => 0,
            Err(LootError::Empty) => 255,
        }
    }

    pub fn message() -> &'static str {
        "game"
    }

    pub fn payload() -> &'static [u8] {
        b"game payload"
    }
}

#[mixin_sdk::targets("demo::skills")]
pub mod skills {
    use core::sync::atomic::{AtomicU32, Ordering::Relaxed};

    static MANA: AtomicU32 = AtomicU32::new(100);
    static EFFECTS: AtomicU32 = AtomicU32::new(0);

    /// The game's documented hook. Mods may adjust the mana cost here.
    fn skill_cost_hook(cost: u32) -> u32 {
        cost
    }

    fn spawn_effect() {
        EFFECTS.fetch_add(1, Relaxed);
    }

    pub fn cast_skill(_skill_id: u32) {
        MANA.fetch_sub(skill_cost_hook(10), Relaxed);
        spawn_effect();
    }

    pub fn mana() -> u32 {
        MANA.load(Relaxed)
    }

    pub fn effect_count() -> u32 {
        EFFECTS.load(Relaxed)
    }

    pub fn skill_power(skill_id: u32) -> i32 {
        skill_id as i32 * 2
    }
}

#[mixin_sdk::targets("demo::targeting")]
pub mod targeting {
    use mixin_sdk::EntityId;

    pub fn find_target(candidate: u32) -> Option<EntityId> {
        EntityId::new(candidate)
    }

    pub fn select_target(candidate: u32) -> Option<EntityId> {
        find_target(candidate)
    }

    pub fn fallback_target(candidate: u32) -> Option<EntityId> {
        EntityId::new(candidate)
    }

    pub fn active_target(id: EntityId) -> EntityId {
        id
    }
}

#[mixin_sdk::targets("demo::physics")]
pub mod physics {
    use mixin_sdk::Vec2;

    pub fn knockback(force: f32) -> Vec2 {
        Vec2 { x: force, y: -1.0 }
    }

    pub fn apply_knockback(force: f32) -> Vec2 {
        knockback(force)
    }

    pub fn vertical_knockback(force: f32) -> Vec2 {
        Vec2 { x: 0.0, y: force }
    }
}

#[cfg(mixin_retained)]
#[repr(C)]
pub struct IntegerVec2 {
    pub x: u32,
    pub y: u32,
}

// A same-sized fixture with different fields for the LLVM signature check.
#[cfg(mixin_retained)]
#[unsafe(no_mangle)]
pub fn integer_knockback(force: f32) -> IntegerVec2 {
    IntegerVec2 {
        x: force.to_bits(),
        y: 0,
    }
}
