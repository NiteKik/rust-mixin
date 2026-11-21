#[mixin_sdk::inject_head("demo::game::play")]
fn before_play() {
    println!("我在这里");
}
