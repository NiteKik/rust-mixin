#![no_std]

use core::ops::ControlFlow;
use example_return_types::LootResult;
use mixin_sdk::{EntityId, Vec2};

#[mixin_sdk::modify_return_result(id = "set_bytes", target = "demo::returns::bytes")]
pub fn set_bytes(mut bytes: [u8; 16]) -> [u8; 16] {
    bytes[0] = 42;
    bytes
}

#[mixin_sdk::modify_return_result(
    id = "increment_bytes",
    target = "demo::returns::bytes",
    priority = 1
)]
pub fn increment_bytes(mut bytes: [u8; 16]) -> [u8; 16] {
    bytes[0] += 1;
    bytes
}

#[mixin_sdk::modify_return_result(id = "scale_vector", target = "demo::returns::vector")]
pub fn scale_vector((x, y): (f32, f32)) -> (f32, f32) {
    (x * 2.0, y + 4.0)
}

#[mixin_sdk::modify_return_result(id = "change_loot", target = "demo::returns::loot")]
pub fn change_loot(value: LootResult) -> LootResult {
    match value {
        Ok(Some(mut loot)) => {
            loot.bytes[0] = 42;
            Ok(Some(loot))
        }
        Err(_) => Ok(None),
        other => other,
    }
}

#[mixin_sdk::modify_return_result(id = "change_message", target = "demo::returns::message")]
pub fn change_message(_: &'static str) -> &'static str {
    "mod"
}

#[mixin_sdk::modify_return_result(id = "change_payload", target = "demo::returns::payload")]
pub fn change_payload(_: &'static [u8]) -> &'static [u8] {
    b"mod payload"
}

#[mixin_sdk::modify_call_result(
    id = "extra_drop",
    target = "demo::loot::drop_count",
    callee = "demo::loot::base_count"
)]
pub fn extra_drop(value: i32) -> i32 {
    value + 1
}

#[mixin_sdk::modify_return_result(id = "invert_gate", target = "demo::loot::gate")]
pub fn invert_gate(value: bool) -> bool {
    !value
}

#[mixin_sdk::modify_parameter(id = "lower_value", target = "demo::loot::narrow", parameter = 0)]
pub fn lower_value(value: i8) -> i8 {
    value.saturating_sub(2)
}

#[mixin_sdk::modify_head(id = "scale", target = "demo::loot::float")]
pub fn scale(value: f32) -> f32 {
    value * -0.5
}

#[mixin_sdk::inject_head(id = "block_skill", target = "demo::skills::cast_skill")]
pub fn block_skill(skill_id: u32) -> ControlFlow<()> {
    if skill_id == 7 {
        ControlFlow::Break(())
    } else {
        ControlFlow::Continue(())
    }
}

#[mixin_sdk::modify_return_result(id = "discount", target = "demo::skills::skill_cost_hook")]
pub fn discount(cost: u32) -> u32 {
    cost / 2
}

#[mixin_sdk::inject_head(id = "override_power", target = "demo::skills::skill_power")]
pub fn override_power(skill_id: u32) -> ControlFlow<i32> {
    if skill_id == 7 {
        ControlFlow::Break(-9)
    } else {
        ControlFlow::Continue(())
    }
}

#[mixin_sdk::inject_head(
    id = "fallback_power",
    target = "demo::skills::skill_power",
    priority = 1
)]
pub fn fallback_power(skill_id: u32) -> ControlFlow<i32> {
    if skill_id <= 8 {
        ControlFlow::Break(-8)
    } else {
        ControlFlow::Continue(())
    }
}

#[mixin_sdk::modify_call_result(
    id = "observe_spawn",
    target = "demo::skills::cast_skill",
    callee = "demo::skills::spawn_effect"
)]
pub fn observe_spawn() {}

#[mixin_sdk::modify_return_result(id = "observe_cast", target = "demo::skills::cast_skill")]
pub fn observe_cast() {}

#[mixin_sdk::modify_return_result(id = "filter_target", target = "demo::targeting::find_target")]
pub fn filter_target(target: Option<EntityId>) -> Option<EntityId> {
    match target.map(EntityId::get) {
        Some(2) => EntityId::new(42),
        Some(3) => None,
        _ => target,
    }
}

#[mixin_sdk::modify_call_result(
    id = "select_target",
    target = "demo::targeting::select_target",
    callee = "demo::targeting::find_target"
)]
pub fn select_target(target: Option<EntityId>) -> Option<EntityId> {
    target.or(EntityId::new(99))
}

#[mixin_sdk::inject_head(id = "override_target", target = "demo::targeting::fallback_target")]
pub fn override_target(candidate: u32) -> ControlFlow<Option<EntityId>> {
    match candidate {
        2 => ControlFlow::Break(EntityId::new(42)),
        3 => ControlFlow::Break(None),
        _ => ControlFlow::Continue(()),
    }
}

#[mixin_sdk::modify_parameter(
    id = "redirect_active_target",
    target = "demo::targeting::active_target",
    parameter = 0
)]
pub fn redirect_active_target(id: EntityId) -> EntityId {
    if id.get() == 2 {
        EntityId::new(42).unwrap_or(id)
    } else {
        id
    }
}

#[mixin_sdk::modify_return_result(
    id = "replace_active_result",
    target = "demo::targeting::active_target"
)]
pub fn replace_active_result(id: EntityId) -> EntityId {
    if id.get() == 42 {
        EntityId::new(84).unwrap_or(id)
    } else {
        id
    }
}

#[mixin_sdk::modify_return_result(id = "scale_knockback", target = "demo::physics::knockback")]
pub fn scale_knockback(value: Vec2) -> Vec2 {
    Vec2 {
        x: value.x * 2.0,
        y: value.y + 4.0,
    }
}

#[mixin_sdk::modify_call_result(
    id = "rotate_knockback",
    target = "demo::physics::apply_knockback",
    callee = "demo::physics::knockback"
)]
pub fn rotate_knockback(value: Vec2) -> Vec2 {
    Vec2 {
        x: -value.y,
        y: value.x,
    }
}

#[mixin_sdk::inject_head(
    id = "override_knockback",
    target = "demo::physics::vertical_knockback"
)]
pub fn override_knockback(force: f32) -> ControlFlow<Vec2> {
    if force < 0.0 {
        ControlFlow::Break(Vec2 { x: 1.0, y: 2.0 })
    } else {
        ControlFlow::Continue(())
    }
}
