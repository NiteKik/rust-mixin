#![no_std]

#[mixin_sdk::modify_call_result(
    id = "bonus_loot",
    target = "demo::loot::drop_count",
    callee = "demo::loot::base_count"
)]
pub fn bonus_loot(count: i32) -> i32 {
    count + 1
}
