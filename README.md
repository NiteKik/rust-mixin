# Rust Mixin

使用普通 Rust 函数和 `#[...]` 属性修改主动接入的游戏组件。游戏与模组由 rustc 生成 LLVM bitcode，加载器在最终优化前定位并变换，再由 LLVM ORC LLJIT 生成机器码。玩家不运行 Cargo/rustc，也不重建游戏发行程序。

当前只有一个根 Cargo workspace。原 `experiments/llvm-mixin` 已提升为主项目，旧 Cranelift、自定义表达式 IR 和 TOML 补丁原型已退出工作区。

## 模组写法

```rust
#[mixin_sdk::modify_call_result(
    id = "half_damage",
    target = "demo::combat::apply_damage",
    callee = "demo::combat::armor_reduce",
    require_matches = 1
)]
pub fn halve(value: f64) -> f64 {
    value * 0.5
}
```

函数体保持普通 Rust，由 rustc 编译。`modify_head` 修改首个参数。`inject_head`、`inject_after_call` 和 `inject_return` 可在返回 `()` 的入口、内部调用和正常返回处执行回调。入口也可通过 `ControlFlow` 提前返回。 [回调写法](docs/DEMO.md)

目标函数支持 0～4 个参数，参数类型可用 `bool`、`f32`、`f64`、8～64 位有符号和无符号整数、`isize`、`usize` 和 `EntityId`。同一个模组可以同时接入游戏预留的 Hook 和注入普通目标函数。签名及编排规则见 [SDK 指南](docs/LLVM_AUTHOR_SDK.md)。

`modify_return_result` 默认用同一套机制处理数字、`()`、实体 ID、`Vec2`、数组、元组、共享结构体和枚举，以及静态引用。游戏与模组使用同一个具体 Rust 类型。调用和示例见 [自定义返回类型](docs/LLVM_AUTHOR_SDK.md#自定义返回类型)。

游戏可以用 `scripts/export-mod-sdk.ps1` 导出作者 SDK。模组作者在独立 Cargo 项目中使用 SDK 的 `build-mod.ps1 -ManifestPath` 打包。参见 [仓库外构建](docs/LLVM_AUTHOR_SDK.md#仓库外构建)。

## 构建与演示

当前受测平台是 Windows x64 MSVC，要求 rustc 1.92.0（ded5c06cf）、LLVM 21.1.3；准备脚本需要 7-Zip。脚本检查编译器身份，stable 已升级时传入 `-Toolchain 1.92.0`。

在项目根目录准备 LLVM，然后运行大象进冰箱演示。

```powershell
pwsh -NoProfile -File scripts/prepare-llvm.ps1
.\demo.cmd
```

原版游戏用 `println!` 依次输出“打开冰箱门”“把大象放进冰箱”“关上冰箱门”。三个模组分别在函数入口、开门调用后和函数结束时输出“我在这里”。构建完成后，可启动 `target/video-demo/video-game.exe --interactive` 在同一进程中切换这三个模组。 [录屏与热加载说明](docs/DEMO.md)

## 目录

```text
crates/
  mixin-sdk/        模组作者接口（no_std，重导出属性宏）
  mixin-macros/     属性宏、目标索引、C ABI 桥接生成
  mixin-runtime/    载荷身份校验、LLVM 定位/变换/优化/JIT 库
  mixin-cli/        只依赖运行时库的加载与调用命令行工具
examples/
  video-game/      录屏用最小游戏与交互式热加载宿主
  mods/
    video-before/   录屏用函数前修改模组
    video-inside/   录屏用函数中修改模组
    video-after/    录屏用函数后修改模组
tests/fixtures/    运行时回归、宿主集成和基准用的测试代码
tools/mixin-bench/ AOT/JIT 差分验证、微基准和内存测量
scripts/           构建、独立模组导出、验证、演示与打包
artifacts/         可重建 bitcode、manifest、索引与补丁计划（忽略）
reports/           历史测量与验证证据，保留原始数据
.tools/            固定 LLVM 开发依赖（忽略）
dist/              可重建演示发行包（忽略）
docs/              当前文档；history/ 保存历史方案和测量说明
```

## 验证、基准和打包

```powershell
pwsh -NoProfile -File scripts/verify.ps1
pwsh -NoProfile -File scripts/benchmark.ps1 -SkipBuild
pwsh -NoProfile -File scripts/package.ps1
```

验证脚本会构建测试组件并运行工作区测试、Clippy 及外部模组验收。基准输出默认在 `reports/current/`。

打包命令将游戏、三个模组和配套的标准库 DLL 放入 `dist/`。运行 `.\dist\demo.cmd` 即可查看演示。

作者用法见 [SDK 指南](docs/LLVM_AUTHOR_SDK.md)，模块职责见 [架构](docs/ARCHITECTURE.md)，验收见 [验证记录](docs/VALIDATION.md)，后续见 [路线图](ROADMAP.md)。
