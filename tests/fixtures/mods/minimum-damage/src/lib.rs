//! A second simple demo mod: change the final result of one game function.
#![no_std]

#[mixin_sdk::modify_return_result(id = "minimum_damage", target = "demo::combat::apply_damage")]
pub fn minimum_damage(value: f64) -> f64 {
    if value < 18.0 { 18.0 } else { value }
}
