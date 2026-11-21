#[mixin_sdk::inject_after_call("demo::game::play", callee = "demo::game::open_fridge")]
fn inside_play() {
    println!("我在这里");
}
