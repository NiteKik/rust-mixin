//! A second, independently compiled mod; no dependency on the game crate.
#![no_std]

#[mixin_sdk::modify_call_result(
    id = "half_damage",
    target = "demo::combat::apply_damage",
    callee = "demo::combat::armor_reduce",
    ordinal = 0,
    require_matches = 1
)]
pub fn halve(value: f64) -> f64 {
    value * 0.5
}
