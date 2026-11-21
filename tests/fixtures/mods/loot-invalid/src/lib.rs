#![no_std]

#[mixin_sdk::modify_call_result(
    id = "wrong_loot_position",
    target = "demo::loot::drop_count",
    callee = "demo::loot::base_count",
    ordinal = 1,
    require_matches = 2
)]
pub fn wrong_loot_position(count: i32) -> i32 {
    count + 10
}
