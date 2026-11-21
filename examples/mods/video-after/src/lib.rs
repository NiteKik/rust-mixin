#[mixin_sdk::inject_return("demo::game::play")]
fn after_play() {
    println!("我在这里");
}
