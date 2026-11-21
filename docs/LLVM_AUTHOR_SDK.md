# LLVM 模组 SDK

库代码在 `crates/`，录屏演示在 `examples/`，验证用的组件和模组在 `tests/fixtures/`。 [已有实现比较](RELATED_WORK.md) · [阶段 A 的历史测量](history/LLVM_FEASIBILITY_RESULTS.md)

## 1. 游戏开发者接入

```rust
#[mixin_sdk::targets("demo::combat")]
pub mod combat {
    pub fn armor_reduce(amount: f64, armor: f64) -> f64 {
        amount - armor * 0.5
    }

    pub fn apply_damage(amount: f64, armor: f64) -> f64 {
        armor_reduce(amount, armor)
    }
}
```

模块内受支持的函数会生成索引。例如 `apply_damage` 的逻辑身份是 `demo::combat::apply_damage`。游戏无需给内部每次调用添加事件或 Hook。

宏保留函数体，交给 rustc 编译。索引边界为普通安全函数，接受 0～4 个参数，支持 `bool`、`f32`、`f64`、`i8` 到 `i64`、`u8` 到 `u64`、`isize`、`usize` 和 `mixin_sdk::EntityId`。返回值可用具体的 `Sized + 'static` 类型。其他签名的辅助函数可写在索引模块之外，再由函数体正常调用。索引函数可以带文档属性。

支持递归处理内联子模块，逻辑路径追加子模块名称；外部文件模块尚无自动遍历能力。当前是宏生成的受限模块索引，不是能识别任意 Rust 项目的 rustc 驱动。

## 2. 模组作者写法

```rust
#[mixin_sdk::modify_call_result(
    id = "adjust_damage",
    target = "demo::combat::apply_damage",
    callee = "demo::combat::armor_reduce",
    ordinal = 0,
    require_matches = 1
)]
pub fn adjust(value: f64) -> f64 {
    let bonus = bonus_for(value);
    (value + bonus) * 0.5
}

fn bonus_for(value: f64) -> f64 {
    if value > 10.0 { 3.0 } else { 0.0 }
}
```

宏分析属性和边界签名，保留函数体并生成桥接和描述。标量修改函数接收一个标量，返回同型标量。计划阶段会比较被修改位置的 Rust 类型，例如 `i32` 位置必须使用 `i32 -> i32`。当目标调用或返回值是 `()` 时，`modify_call_result` 和 `modify_return_result` 可接受 `fn()` 观察回调，分别在原调用完成后和原有返回出口前执行。

只执行回调时，可用 `inject_after_call` 在一个返回 `()` 的内部调用之后执行 `fn()`，或用 `inject_return` 在目标函数正常返回 `()` 前执行 `fn()`。这两个属性和 `inject_head` 都接受第一个字符串参数作为目标路径。各处理属性省略 `id` 时默认使用 `crate名::函数名`，同一 crate 中有同名处理函数时可显式指定不同的 ID。内部调用选择默认使用 `ordinal = 0` 和 `require_matches = 1`。 [冰箱演示](DEMO.md)

回归测试中的调整模组包含结构体、局部借用和分支。调整模组和减半模组都只依赖 SDK。 [调整模组](../tests/fixtures/mods/adjust-damage/src/lib.rs) · [减半模组](../tests/fixtures/mods/half-damage/src/lib.rs)

普通 Rust 的局部借用检查仍生效。检查脚本用一个失效局部引用的模组验证 rustc 报 E0597。

除了 `modify_call_result`，当前 SDK 还支持修改目标函数的正常返回值：

```rust
#[mixin_sdk::modify_return_result(
    id = "minimum_damage",
    target = "demo::combat::apply_damage"
)]
pub fn minimum_damage(value: f64) -> f64 {
    if value < 18.0 { 18.0 } else { value }
}
```

SDK 在目标函数的正常返回路径生成一个调用点。模组的处理函数接收原结果并返回同类型的新结果。

还可以在目标函数入口修改一个参数：

```rust
#[mixin_sdk::modify_parameter(
    id = "bonus_amount",
    target = "demo::combat::apply_damage",
    parameter = 0
)]
pub fn bonus_amount(value: f64) -> f64 {
    value + 4.0
}
```

`parameter = 0` 表示第一个参数。处理函数使用被选中参数的标量类型。加载器在函数入口的第一条指令前生成处理调用，并让目标函数使用修改后的参数。

如果注入点就是函数入口的第一个参数，也可以使用更接近 Mixin 术语的 `modify_head`：

```rust
#[mixin_sdk::modify_head(
    id = "head_bonus",
    target = "demo::combat::apply_damage"
)]
pub fn head_bonus(value: f64) -> f64 {
    value + 4.0
}
```

它在目标函数的第一条 LLVM 指令前处理第一个参数。目标必须至少有一个参数，处理函数使用首个参数的标量类型。

### 可取消的入口回调

`inject_head` 的参数类型和顺序与目标函数一致。目标返回 `()` 且回调始终继续执行时，可将回调写成返回 `()` 的普通函数。需要提前返回时，回调使用 `ControlFlow<T>`，其中 `T` 是目标的返回类型。下面的目标签名为 `cast_skill(u32) -> ()`。

```rust
use core::ops::ControlFlow;

#[mixin_sdk::inject_head(id = "block_skill", target = "demo::skills::cast_skill")]
pub fn block_skill(skill_id: u32) -> ControlFlow<()> {
    if skill_id == 7 {
        ControlFlow::Break(())
    } else {
        ControlFlow::Continue(())
    }
}
```

入口回调按 `priority` 和通常的同优先级规则执行，读取原始参数。入口回调全部继续执行时，才会应用参数修改并执行原函数体。第一个 `Break(value)` 会让目标立即返回该值，跳过后续回调、原函数体和正常返回处的处理函数。

### 实体 ID 和可选目标

`mixin_sdk::EntityId` 按值传递一个非零的 `u32`。用 `EntityId::new(number)` 构造 ID，用 `id.get()` 读取数值。游戏负责解释 ID 及检查实体是否存在。跨界时 `Option<EntityId>` 用零表示 `None`，其余值表示 `Some(id)`。补丁计划分别核对 `u32`、`EntityId` 和 `Option<EntityId>`。

目标函数可返回 `Option<EntityId>`。返回值修改和调用结果修改的处理函数使用 `Option<EntityId> -> Option<EntityId>`。入口回调使用 `ControlFlow<Option<EntityId>>`，其中 `Break(None)` 表示立即返回无目标，`Continue(())` 会让原函数继续执行。

```rust
use mixin_sdk::EntityId;

#[mixin_sdk::modify_return_result(id = "filter_target", target = "demo::targeting::find_target")]
pub fn filter_target(target: Option<EntityId>) -> Option<EntityId> {
    match target.map(EntityId::get) {
        Some(2) => EntityId::new(42),
        Some(3) => None,
        _ => target,
    }
}
```

CLI 显示 `Some(EntityId(42))` 或 `None`。JSON 报告对应输出数字或 `null`，目标签名记录返回类型。实体 ID 参数可以用 `--arg 42` 传入。

### 二维向量返回值

游戏和模组共用 `mixin_sdk::Vec2`，即含两个 `f32` 字段 `x`、`y` 的 `#[repr(C)]` 类型。目标函数可返回 `Vec2`。返回值及调用结果修改的处理函数使用 `Vec2 -> Vec2`，入口注入可以用 `ControlFlow<Vec2>` 提前返回。构建时检查类型的大小、对齐和字段偏移，计划阶段还会核对 LLVM 的返回类型和处理函数签名。

```rust
use mixin_sdk::Vec2;

#[mixin_sdk::modify_return_result(id = "scale_knockback", target = "demo::physics::knockback")]
pub fn scale_knockback(value: Vec2) -> Vec2 {
    Vec2 { x: value.x * 2.0, y: value.y + 4.0 }
}
```

目标函数的参数仍采用前述标量和实体 ID 类型。JSON 报告将向量写为包含 `x` 和 `y` 的对象。

### 自定义返回类型

游戏把目标函数放进 `#[mixin_sdk::targets(...)]` 模块，模组用 `#[mixin_sdk::modify_return_result(...)]` 指定目标。标准类型和自定义类型默认使用同一个返回值调用点。目标参数使用 SDK 支持的标量类型，处理函数接收并返回目标的具体 Rust 类型。

```rust
// 游戏 crate
#[mixin_sdk::targets("demo::returns")]
pub mod returns {
    pub fn bytes(seed: u8) -> [u8; 16] {
        [seed; 16]
    }
}

// 独立的模组 crate
#[mixin_sdk::modify_return_result(id = "set_bytes", target = "demo::returns::bytes")]
pub fn set_bytes(mut bytes: [u8; 16]) -> [u8; 16] {
    bytes[0] = 42;
    bytes
}
```

普通返回、提前 `return` 和通过 `?` 正常返回都会经过同一个调用点。`ordinal` 默认且只接受 0，`require_matches` 默认且只接受 1。多个模组沿用原有排序规则，后一个模组接收前一个写回的值。

返回类型须为具体的 `Sized + 'static` 类型。游戏和模组用固定工具链编译，并从同一个依赖 crate 导入自定义类型。Rust 编译器生成结果读写代码，加载器统一传递临时结果的地址。计划预览验证注入位置和入口签名。JIT 生成后、交给宿主前，加载器会比较游戏和模组的 Rust `TypeId`，类型不符时返回错误。

用例覆盖了 `[u8; 16]`、`(f32, f32)`、共享的 `Result<Option<Loot>, LootError>`、`&'static str` 和 `&'static [u8]`。宿主可用 `unsafe Jit::call_typed::<T>` 取得结果，并负责让 JIT 在返回值引用的代码和数据使用期间保持存活。游戏也可以像 `demo::returns::loot_first_byte` 一样，在 JIT 内调用返回自定义类型的目标，再将所需字段通过标量入口交给宿主或 CLI。冰箱演示通过固定工具链的标准库 DLL 验证了 `println!`。跨 JIT 生命周期传递堆容器仍需单独验证。 [录屏演示](DEMO.md)

升级时先用当前 SDK 重新构建游戏 bitcode，再为该游戏版本重新打包模组。原有 `typed_return` 源码标记可以省略。

### 与游戏预留 Hook 一起使用

游戏可把预留 Hook 实现成组件中有默认行为的索引函数。示例中的 `cast_skill` 原本就调用 `skill_cost_hook(cost: u32) -> u32`，默认返回原价。同一个 Cargo 模组既能用 `#[modify_return_result]` 修改这个 Hook 的结果，也能用 `#[inject_head]` 在 `cast_skill` 入口添加拦截。两个处理函数编入同一个模组包，游戏在启动时一起加载。

### 订阅游戏宿主的原生事件

战斗宿主另有一个 `on_damage` 事件，在 AOT 或 JIT 计算出伤害之后触发。`plugin.dll` 的 `mixin_register_v1` 入口接收宿主提供的注册函数，订阅 `extern "C" fn(f64) -> f64` 回调。宿主按注册顺序传递伤害值，并在使用回调期间持有 DLL。这个事件及其 ABI 由示例游戏定义。

`tests/fixtures/mods/damage-plugin` 在同一个 Cargo 工程中包含事件订阅和 Mixin，使用 `scripts/build-damage-plugin.ps1` 生成测试包。只加载 DLL 时，战斗组件走 AOT，回调无需 LLVM。有 `code.bc` 的包会在启动时应用 Mixin，随后由同一宿主事件处理结果。DLL 和 bitcode 分别编译，各自持有自己的静态变量。

## 3. 构建与独立加载

```powershell
# 首次准备运行库并构建测试组件、SDK 和 CLI
pwsh -NoProfile -File scripts/prepare-llvm.ps1
pwsh -NoProfile -File scripts/build.ps1

# 单独构建模组。不会构建游戏组件或运行器。
pwsh -NoProfile -File scripts/build-mod.ps1 -Package llvm-handler
pwsh -NoProfile -File scripts/build-mod.ps1 -Package llvm-half-mod

# 同一个已构建的运行器加载不同模组。
.\target\release\mixin-cli.exe --call demo::combat::apply_damage --mod artifacts/mods/llvm-handler --report reports/current/manual-adjust.json
.\target\release\mixin-cli.exe --call demo::combat::apply_damage --mod artifacts/mods/llvm-half-mod --report reports/current/manual-half.json
```

从工作区根目录执行。结果分别为 `16 → 9.5` 和 `16 → 8`。单独运行 `plan` 子命令可检查目标和匹配数量，并输出补丁计划。

```powershell
.\target\release\mixin-cli.exe plan --mod artifacts/mods/llvm-handler --report reports/current/manual-plan.json
```

### 仓库外构建

游戏发布者从已构建的游戏产物导出 SDK。掉落示例使用 `artifacts/scalar`，战斗示例使用默认的 `artifacts`。

```powershell
.\scripts\export-mod-sdk.ps1 -GameArtifacts artifacts/scalar -Output D:\LootSdk
```

作者在自己的 Cargo 库项目中声明 SDK 路径依赖。游戏发布者可以同时提供逻辑目标和注入位置说明。以下是掉落模组所需的依赖和处理函数。

```toml
[dependencies]
mixin-sdk = { path = "D:/LootSdk/crates/mixin-sdk" }
```

```rust
#![no_std]

#[mixin_sdk::modify_call_result(
    id = "bonus_loot",
    target = "demo::loot::drop_count",
    callee = "demo::loot::base_count"
)]
pub fn bonus_loot(count: i32) -> i32 {
    count + 1
}
```

用固定工具链生成锁文件，再按 Cargo.toml 中的包名构建。SDK 默认读取自己附带的游戏产物。

```powershell
rustup run 1.92.0 cargo generate-lockfile --manifest-path .\Cargo.toml
& 'D:\LootSdk\scripts\build-mod.ps1' -Toolchain 1.92.0 -ManifestPath .\Cargo.toml -Package my-loot-mod -Output .\dist\my-loot-mod
```

SDK 随游戏构建发布它实际使用的共享 crate 编译产物，`build-mod.ps1 -ManifestPath` 会按对应的 crate 名让模组编译器使用这些产物。玩家只需要游戏包和模组包。

游戏可以在生成基础 bitcode 时，用 `Export-RetainedBitcode -SharedCrates` 选择要共享身份的 crate。例如掉落示例发布 `mixin_sdk` 和 `example_return_types`。发布者同时提供这些 crate 的 Cargo 源码依赖，作者在自己的 `Cargo.toml` 中按相同 crate 名声明。这样游戏和模组会得到相同的 Rust `TypeId`，包括 `Vec2` 和 `Result<Option<Loot>, LootError>`。加载器仍会拒绝把另行定义、布局相同的结构体当作游戏返回类型。

此前只含源码的 SDK 可以继续用于已支持的基础类型。要共享由 crate 定义的返回类型，用当前构建脚本重新生成游戏产物并导出 SDK，再用该 SDK 构建模组。

`build-mod.ps1` 的构建输出包含以下文件。

```text
code.bc       rustc 生成的代码及编译进去的属性描述
code.ll       作者检查用的文本 IR，玩家加载不需要
mod.json      构建身份、基础游戏指纹和代码哈希
```

`mod.json` 只描述兼容条件，不承载手写函数。作者写 Rust，工具构建产物，运行器加载真实 LLVM 代码。游戏基础版本使用 bitcode 精确 SHA-256 绑定，版本不符拒绝。

向玩家分发时携带 `mod.json` 和 `code.bc` 即可。

作者保存源码后重新构建模组包。玩家可以在启动时选用模组包，接入 `HotReload` 的宿主也可在运行中主动切换。

多个模组：

```powershell
.\target\release\mixin-cli.exe --call demo::combat::apply_damage --mod artifacts/mods/llvm-handler --mod artifacts/mods/llvm-half-mod --report reports/current/manual-both.json
```

先调整再减半得到 4.75，交换两个 `--mod` 的顺序得到 4。执行模式要求用 `--call <逻辑身份>` 指定目标。用重复的 `--arg <值>` 传入参数。两个 `f64` 参数默认是 20 和 8，也可用 `--arg0` 和 `--arg1` 分别覆盖。

## 4. 索引与保留机制

作者不必知道 LLVM mangled name 或内存地址。编译宏为每个逻辑身份生成：

- 基于逻辑身份哈希的函数符号。
- 明确 C ABI 的纯值入口桥接。
- 带 schema 的常量字节描述，记录逻辑身份、实际符号、入口、参数数量和保留标记。

在 `mixin_retained` 构建中，宏给目标函数加工具生成的 noinline 标记，避免目标调用在 rustc 前段就被内联掉。默认 AOT 构建没有这个标记。

加载器从真实 bitcode 的常量读取描述，验证名字、签名和 schema，建立逻辑索引；它不再写死 damage/armor 的实际符号。选择器解析完成后，加载器只移除有明确来源的工具保留标记，再执行 LLVM O3。当前不接受用户自己给被索引函数添加 inline 属性，避免错误移除用户意图。

这没有保证任意源码结构都保留：部分 MIR 简化、间接调用、动态分发、宏展开等仍有自己的语义。当前只在实际保存的代码上选择可识别的直接调用；失配就拒绝。

CLI 通过运行时索引查找命令行传入的目标。基准程序使用战斗测试组件的入口。运行时在加载时读取描述和解析补丁位置。 [CLI 实现](../crates/mixin-cli/src/main.rs) · [基准入口](../tools/mixin-bench/src/fixture.rs)

CLI 的 `--report` 指定 JSON 报告路径。报告包含逻辑索引和补丁计划。执行模式还生成同名的 `.before.ll` 和 `.optimized.ll`，分别记录变换后优化前的 IR 和优化后的 IR。验证程序另将索引和计划写入测试组件的产物目录。 [验证入口](VALIDATION.md)

## 5. 选择和组合规则

所有补丁先对不可变的基础代码解析位置，全部验证通过后再变换。新插入的处理函数调用不参与原始 ordinal 编号。

- 调用结果操作用 `require_matches` 指定直接调用的精确总数，用 `ordinal` 从 0 开始选择一个调用点。顺序取自 LLVM 基本块和指令遍历。
- 返回值操作固定在统一的返回调用点，默认 `require_matches = 1` 和 `ordinal = 0`。参数操作通过参数索引选择位置。
- `priority`：当前允许 0..=i32::MAX，默认 0，数值小的先执行。
- 同优先级：按 --mod 输入顺序，再按每个模组内 handler ID 字典序。
- 同位置依次组合为 B(A(original()))，原调用只执行一次。
- 重复 handler ID、目标缺失、匹配数量错误、ordinal 越界和 ABI 不符拒绝。

使用 `HotReload` 的单线程宿主会在候选构建成功后才切换 JIT。候选失败时，后续调用继续使用上一套规则。 [生命周期约定](ARCHITECTURE.md#生命周期和限制)

## 6. 使用边界

局部变量捕获、通用游戏对象跨界、泛型目标、共享状态、标准库通用链接、多线程切换和稳定发行包仍需后续支持。随调用借用的返回值和 `impl Trait` 返回值仍需其他接口。

LLVM DLL 仍按进程期保留；显式卸载后的退出依赖未审计完。bitcode 是可信本机代码，身份/哈希和 LLVM verifier 都不构成沙箱。

