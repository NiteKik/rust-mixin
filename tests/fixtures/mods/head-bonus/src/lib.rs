//! A code Mixin that adjusts the first target argument before its body runs.
#![no_std]

#[mixin_sdk::modify_head(id = "head_bonus", target = "demo::combat::apply_damage")]
pub fn head_bonus(value: f64) -> f64 {
    value + 4.0
}
