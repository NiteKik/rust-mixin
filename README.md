# Rust Mixin

Rust Mixin 为 Rust 游戏提供模组支持。游戏开发者开放可修改的组件，模组作者用普通 Rust 函数和属性选择要修改的函数或其中的调用位置。游戏加载模组时，运行时用 LLVM 将修改应用到组件的 bitcode，再生成可执行代码。玩家可以用同一份游戏切换模组。

## 模组示例

这段代码会在 `play` 中的 `open_fridge` 调用结束后输出“我在这里”。[完整示例](examples/mods/video-inside/src/lib.rs)

```rust
#[mixin_sdk::inject_after_call("demo::game::play", callee = "demo::game::open_fridge")]
fn inside_play() {
    println!("我在这里");
}
```

## 运行演示

源码演示需要 Windows x64（MSVC）、Rust 1.92.0、PowerShell 7 和 7-Zip。在仓库根目录运行以下命令。准备脚本会下载并提取 LLVM 21.1.3。

```powershell
pwsh -NoProfile -File .\scripts\prepare-llvm.ps1
.\demo.cmd
```

`demo.cmd` 默认使用 rustup 的 `stable` 工具链。如果 `stable` 已更新，安装 Rust 1.92.0 后运行 `.\demo.cmd -Toolchain 1.92.0`。

演示先运行原版的“大象进冰箱”，再依次加载三个模组，分别在开门前、开门后和关门后输出“我在这里”。构建后可运行 `.\target\video-demo\video-game.exe --interactive`，在同一进程内切换模组。[演示与热切换步骤](docs/DEMO.md)

## 文档

- [SDK 指南](docs/LLVM_AUTHOR_SDK.md)
- [运行时架构](docs/ARCHITECTURE.md)
- [验证、基准与打包](docs/VALIDATION.md)
- [路线图](ROADMAP.md)
