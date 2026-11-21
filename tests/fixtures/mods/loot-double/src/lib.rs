#![no_std]

#[mixin_sdk::modify_return_result(id = "double_loot", target = "demo::loot::drop_count")]
pub fn double_loot(count: i32) -> i32 {
    count * 2
}
