# 类似项目调研

核对日期：2026-11-20。通过 GitHub 仓库搜索、项目 README 和使用文档核对；原始快照保存在 `docs/research/2026-11-20/`，包含来源 URL、README blob SHA 和抓取时间。没有安装或运行这些项目，没有独立复现其测试数量或性能宣称。

结论：确实有 Rust 源码 Mixin、原生热重载、函数热补丁和完整 Rust JIT 项目。它们覆盖不同部分。本次检索未找到已经验证可直接替代本项目全部目标的现成方案；这不是“全网不存在”的结论。

## 1. 最接近作者功能：horizon-mixins

来源：[horizon-rust/horizon-mixins](https://github.com/horizon-rust/horizon-mixins)。MIT。

上游 README 描述的主线：索引原始 Cargo workspace -> 验证组合计划 -> 变换 Rust AST -> 独立 staging workspace -> Cargo 编译/检查。

文档列出 HEAD、RETURN、OVERWRITE、调用前后、参数/变量/表达式修改、局部捕获和 accessor/invoker 等能力，并说明了 typed helper、冲突与顺序规则以及原始源码保护。

它修改源码并重新构建目标。Rust Mixin 在玩家进程中加载保留的 LLVM bitcode，应用经验证的补丁并生成可调用的机器码。 [架构](ARCHITECTURE.md)

可借鉴：从不可变原始代码解析全部选择器、精确匹配数量、明确组合顺序、保守拒绝不确定位置、保持原始工程不变、生成可审查的计划。源变换之后重新交给 rustc 检查也有自己的安全优势，不能说运行时 LLVM 路线天然更安全。

## 2. 真实 Rust 动态库热重载：hot-lib-reloader

来源：[rksm/hot-lib-reloader-rs](https://github.com/rksm/hot-lib-reloader-rs)。MIT。

通过 libloading 和宏包装加载 Rust dylib，开发者重新构建库后可以让程序继续使用新函数。文档要求可热重载函数有可找到的导出名称，并讨论签名、类型布局、泛型导出、全局状态和 TypeId 变化。

它解决动态库边界内的函数更新，不自动提供“选择现有函数内部某次调用”的 Mixin 选择器。跨界类型也不能自由改变；热重载不能代替 ABI/对象寿命设计。

可借鉴：重载等待/观察机制、库生命周期、状态所有权、签名不变的边界。若另走原生 DLL 模组分支，可进一步评估直接采用；当前 LLVM bitcode 变换路径不由它直接替代。

## 3. 原生函数热补丁：hot-rust

来源：[humdrum00001010/hot-rust](https://github.com/humdrum00001010/hot-rust)。本轮 API 未提供可直接确认的 SPDX 许可，采用代码前需另查 LICENSE。

README 将其标为 alpha，已测试目标为 macOS Apple Silicon、主要面向 debug/dev。通过 Cargo 包装和 rust-analyzer 观察修改，构建小型补丁，再安装到正在运行的进程；使用不稳定的 patchable-function-entry 编译标志。

文档限定单函数体修改，签名、类型、布局、trait、宏或多函数修改需要重启，并提醒优化内联可能影响工作。要求作者侧 Rust/rust-analyzer，不是已验证的玩家免工具链内部注入方案。

它与后续函数级发布/替换很相关，但不是任意内部指令位置的选择与组合。可研究其编译包装、入口身份和发布验证思路，不能将其当前平台支持套用为 Windows 支持。

## 4. 机器码入口 Hook：retour

来源：[Hpmason/retour-rs](https://github.com/Hpmason/retour-rs)。具体许可证需以仓库 LICENSE 和依赖为准，本轮未做完整许可审计。

提供原生 detour/trampoline，处理指令搬移、相对地址等问题，可以把一个函数入口转到另一实现，并保留调用原函数的途径。README 说明 static-detour 需要 nightly，并讨论修改时其他线程正在执行入口指令的问题。

它不保留 Rust 的源码结构，不能自然提供“第三次调用”“局部变量”“私有字段”这些高级选择器，也不能恢复被优化掉的调用。适合评估整个函数入口接管，不能单靠它完成目标中的 Java 风格 Mixin。

## 5. 完整 Rust 的 JIT 后端：rustc_codegen_cranelift

来源：[rust-lang/rustc_codegen_cranelift](https://github.com/rust-lang/rustc_codegen_cranelift)，[JIT 使用文档](https://github.com/rust-lang/rustc_codegen_cranelift/blob/master/docs/usage.md)。MIT/Apache-2.0。

它复用 rustc 前端，通过 Cranelift 代码生成。官方使用文档有 cargo-clif jit / jit-mode，说明能够执行真实 Rust，且明确称该 JIT 模式高度实验性、依赖准备可能困难，并要求依赖以动态库形式可用。

这证明“完整 Rust + JIT”不必从头实现 Rust。它没有直接提供本项目需要的模组选择器和目标程序变换层；将编译器 JIT 模式直接交给玩家，也不能满足当前分发和依赖目标。

## 6. 可复用 LLVM 封装：Inkwell

来源：[TheDan64/inkwell](https://github.com/TheDan64/inkwell)。Apache-2.0。

README 描述为 llvm-sys 上的更强类型 Rust 包装，并列出 LLVM 12～23 的版本特性，覆盖本实验的 LLVM 21。它可以减少部分手写 C API 和裸指针使用；JIT 函数查找/调用依然涉及签名与不安全代码边界。

它是编译基础设施，不是 Mixin 引擎。README 中的 JIT 示例也不足以证明我们使用的全部 ORC、动态 DLL 装载和所有权接口可以直接迁移。

## 7. 自带热重载的语言：Mun

来源：[mun-lang/mun](https://github.com/mun-lang/mun)。README 声明 Runtime 为 MIT/Apache-2.0。

Mun 设计了自己的静态类型语言、LLVM AOT 编译和运行时，热重载是其核心能力，包含自己的对象/GC 规则。可以研究代码身份、依赖和对象更新策略；直接采用它需要改变模组语言，不能描述为正常 Rust Mixin 的现成实现。

## 8. 对照规范：Java Mixin

来源：[SpongePowered/Mixin](https://github.com/SpongePowered/Mixin)。MIT。

官方 README 明确描述 ASM 与类加载过程。它是选择器、注入、冲突诊断的主要参考，不是 Java 程序任何时刻都能任意热修改的证明。表达式结果修改等能力还涉及 MixinExtras 或其他扩展，不能把相似属性名称宣传为完整兼容。

## 9. 本项目的取舍

| 需要的部分 | 参考或复用对象 | 当前决定 |
| --- | --- | --- |
| Rust 函数语言、局部类型与借用检查 | rustc | 已直接复用 |
| IR 链接、优化、JIT | LLVM ORC | 已直接复用 |
| 选择器和注入契约 | Java Mixin、horizon-mixins | 借鉴契约，仍按 Rust 边界单独验证 |
| 动态发布、代码与状态寿命 | hot-lib-reloader、hot-rust、Mun | `HotReload` 在单线程宿主的调用间发布新 JIT，宿主持有持久状态 |
| LLVM C API 绑定 | llvm-sys / Inkwell | 运行时使用自身的 C API 绑定 |
| 任意未准备二进制的入口接管 | retour | 不纳入本轮 LLVM 主线 |

属性、选择器及组合规则见 [SDK 指南](LLVM_AUTHOR_SDK.md)。
