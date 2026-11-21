# LLVM Mixin 架构

本项目只有根工作区一条实现路线。游戏构建时保留真实 rustc LLVM bitcode；作者使用 Rust 属性函数描述修改；运行时完成验证、链接、变换、优化和 JIT。

## 模块职责

- `crates/mixin-sdk`：no_std 作者入口，重导出宏，无运行时加载依赖。
- `crates/mixin-macros`：解析属性和签名，保留函数体；生成逻辑 ID、符号、C 桥接及常量描述。
- `crates/mixin-runtime/src/bundle.rs`：检查工具链、目标平台、基础代码 SHA-256 和模组载荷哈希。
- `crates/mixin-runtime/src/metadata.rs`：目标索引、操作枚举、字段约束及补丁计划。
- `crates/mixin-runtime/src/llvm.rs`：LLVM C API 绑定、链接、定位、变换、O3、ORC LLJIT 和代码所有权。
- `crates/mixin-cli`：解析命令行，使用公开运行时库加载任意符合当前边界的组件与模组，不链接演示游戏。
- `examples/video-game` 和 `examples/mods/video-before`、`video-inside`、`video-after`：录屏用最小游戏、交互式热加载宿主及三个位置模组。
- `tests/fixtures` 包含运行时回归、宿主集成和基准使用的测试组件与模组。
- `tools/demo-build` 将 Cargo 构建别名转给演示构建脚本。
- `tools/mixin-bench` 对比 AOT 与 JIT 的结果，并测量执行、加载、热切换和 Windows 进程内存。

运行时测试读取 `scripts/build.ps1` 生成的测试 bitcode。运行时与 SDK 共用 `EntityId` 和 `Vec2`。CLI 用 `--call` 指定逻辑 ID，通过重复的 `--arg` 按目标签名解析参数。

## 执行流程

```text
游戏 Rust + #[targets]            模组 Rust + #[modify_*] 或 #[inject_*]
           ↓ rustc                        ↓ rustc
基础 bitcode + manifest          模组 code.bc + mod.json
                    ↓ mixin-runtime
    校验身份/哈希 → 读取索引 → 链接 → 解析全部原始位置
                    ↓
    按优先级变换 → verifier → 移除工具保留屏障 → LLVM O3
                    ↓
             ORC LLJIT → 完整物化入口 → 调用
```

`mod.json` 记录工具链、目标平台、游戏 bitcode 指纹和模组载荷哈希。函数逻辑与属性生成的补丁描述位于 `code.bc`。

所有选择器在不可变基础代码上解析，新增 handler 调用不参与原始 ordinal 编号。同位置按 priority 升序、输入模组顺序、模组内 ID 字典序组合。调用结果保留一次原始求值。参数入口变换与返回出口变换有不同的位置规则，详见 SDK。

## 生命周期和限制

`Jit` 持有代码内存，在单线程同步调用，不能 Send/Sync。`call` 逐个检查参数类型，`call_two` 支持双 `f64` 调用。通用标量入口通过固定签名的 C ABI 传递位表示，双 `f64` 入口按原签名调用。底层 `lookup_raw` 为 unsafe，调用者负责准确 ABI、线程限制和 JIT 的存活。

`HotReload` 为单线程宿主提供切换接口。宿主在两次组件调用之间提交完整、有序的模组目录列表。加载器从基础 bitcode 编译候选并物化入口，成功后替换当前 JIT 并释放旧 JIT。候选失败时当前 JIT 继续服务。清空列表恢复原版规则。

`HotReload::background` 在宿主的 `std::thread::scope` 内运行。每次 `request` 启动一个后台准备任务，负责读包、校验、编译和入口物化。宿主照常在自己的线程执行当前规则。宿主在两次调用之间用 `try_activate` 检查任务，接收就绪结果并释放上一代 JIT。待处理任务上限为一个，结束作用域时会等待后台任务退出。跨线程交接限于准备完成的 JIT，游戏入口由宿主独占调用。

热切换入口使用可跨切换保留的标量值，持久游戏状态由宿主持有。模组需遵守作用域约定，让函数指针、借用和后台工作在该次调用内结束。战斗测试宿主另有独立的原生 DLL 事件加载接口。 [热切换实现](../crates/mixin-runtime/src/hot_reload.rs) · [事件加载实现](../tests/fixtures/embedded-game/src/plugins.rs)

LLVM DLL 保留到进程结束，各 JIT 实例独立释放。LLVM verifier 校验 IR 结构。加载器额外用元数据比较 Rust 标量类型，例如 `i32` 和 `u32` 在 LLVM 中都表示为 `i32`，计划阶段仍会拒绝类型不符的处理函数。模组是可信原生代码。宿主可用 `HotReload::with_libraries` 为每一代 JIT 指定提供导入符号的 DLL，冰箱演示用该接口链接固定工具链的 Rust 标准库。

## 数据与历史

`artifacts/`、`.tools/`、`target/`、`dist/` 为可重建数据。`reports/` 保存验证和测量记录。`docs/history/` 保存历史方案和实验报告，记录使用当时的路径和程序名。
