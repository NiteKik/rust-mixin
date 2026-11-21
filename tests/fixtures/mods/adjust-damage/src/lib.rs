//! Separate mod code. Locally borrowed structs and helpers are compiled by rustc.
#![no_std]

struct Rule {
    threshold: f64,
    bonus: f64,
    multiplier: f64,
}

fn bonus_for(value: f64, rule: &Rule) -> f64 {
    if value > rule.threshold {
        rule.bonus
    } else {
        0.0
    }
}

#[mixin_sdk::modify_call_result(
    id = "adjust_damage",
    target = "demo::combat::apply_damage",
    callee = "demo::combat::armor_reduce",
    ordinal = 0,
    require_matches = 1
)]
pub fn mod_adjust_damage(value: f64) -> f64 {
    let rule = Rule {
        threshold: 10.0,
        bonus: 3.0,
        multiplier: 0.5,
    };
    let bonus = bonus_for(value, &rule);
    (value + bonus) * rule.multiplier
}
