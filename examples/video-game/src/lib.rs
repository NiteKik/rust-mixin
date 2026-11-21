#[mixin_sdk::targets("demo::game")]
pub mod game {
    fn open_fridge() {
        println!("打开冰箱门");
    }

    pub fn play() {
        open_fridge();
        println!("把大象放进冰箱");
        println!("关上冰箱门");
    }
}
