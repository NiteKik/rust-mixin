//! A simple parameter Mixin: add four points to the incoming damage amount.
#![no_std]

#[mixin_sdk::modify_parameter(
    id = "bonus_amount",
    target = "demo::combat::apply_damage",
    parameter = 0
)]
pub fn bonus_amount(value: f64) -> f64 {
    value + 4.0
}
