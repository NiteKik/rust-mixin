# 验证记录

## 环境与入口

Windows x64 MSVC，rustc 1.92.0 commit ded5c06cf，LLVM 21.1.3。依赖由根 Cargo.lock 固定。

```powershell
pwsh -NoProfile -File scripts/verify.ps1
```

验收脚本检查格式、构建测试组件和模组的 bitcode，并执行工作区 Release 测试和 `Clippy -D warnings`。随后检查以下行为。 [验收脚本](../scripts/verify.ps1)

- AOT 与 JIT 结果一致，覆盖有限值、负零、无穷大、NaN、分支和批量循环。
- 独立模组在空 `PATH` 的进程中能修改调用结果、返回值、参数和入口行为。组合结果随模组顺序变化。错误目标、位置或签名会被拒绝。单独构建模组后，基础 bitcode 和 CLI 的哈希保持一致。
- 作者编译时拒绝失效的局部引用、非法 `ordinal` 和入口注入不支持的选择器。
- 战斗宿主能加载内嵌 bitcode、原生 DLL 事件和组合模组。独立掉落宿主能启动加载和运行中切换。

差分报告写入 `reports/validation.json`，外部模组验收报告写入 `reports/current/external-*.json`。

独立掉落宿主的验收由 `scripts/check-loot-host.ps1` 完成。它用同一个发布包验证启动加载、错误选择器诊断及进程内切换，覆盖两种模组替换、失败后保留旧规则、恢复原版、持久状态连续及 32 次重复组合与卸载。

掉落验收先发布宿主和游戏 bitcode，再用导出的 SDK 在仓库外构建独立 Cargo 项目。额外校验共享编译身份的 `Vec2`、游戏定义的 `LootResult`，以及对布局相同的错误类型的拒绝。脚本在 `target/loot-host-<id>` 生成验收包，复制并运行包内的 `check-loot-player.ps1`，以空 `PATH` 验证发布包。将整份验收包复制到未安装 Rust 的 Windows x64 环境后，可在包目录中运行以下命令。

```powershell
powershell -NoProfile -File .\check-loot-player.ps1 -RequireNoRust
```

## 验收录屏演示包

在工作区根目录生成演示包，再用包内脚本运行已编译的“大象进冰箱”游戏和三个位置模组。 [交互式加载步骤](DEMO.md)

```powershell
pwsh -NoProfile -File scripts/package.ps1
.\dist\demo.cmd
```

## 执行性能测量

基准脚本默认运行三个独立进程，每种对比采样 21 次，每个样本循环 1,600 万次，将结果及 IR 写到 `reports/current`。

```powershell
pwsh -NoProfile -File scripts/benchmark.ps1
```

早期 AOT/JIT 样本保存在 `reports/run-*.json`、`reports/phase-b/run-*.json` 和相应 CSV。 [阶段 A 历史报告](history/LLVM_FEASIBILITY_RESULTS.md) · [阶段 B 原始样本](../reports/phase-b)

## 单线程热切换测量

```powershell
pwsh -NoProfile -File scripts/benchmark-reload.ps1
```

脚本对实际掉落组件和生成的较大组件各启动三个独立进程。每个进程先预热 16 轮，再测量 128 轮启用双模组与恢复原版。每次切换后核对掉落为 8 或 3。计时覆盖包读取、JIT 编译、切换和旧 JIT 释放。进程私有内存每 16 轮在恢复基础规则后采样一次。

以下为归档样本的结果。

| 组件 | 启用中位数 | 启用 p95 | 恢复原版 p95 |
| --- | --- | --- | --- |
| 掉落组件，33 个目标 | 29.86–30.48 ms | 33.47–36.76 ms | 31.03–33.29 ms |
| 合成组件，98 个目标 | 270.58–273.94 ms | 296.95–364.37 ms | 288.16–334.33 ms |

结果来自本机 Windows x64 和 i7-13620H。全部 1,536 次实测切换超过 60 FPS 的单帧时长。掉落组件的私有内存约为 17 MiB。合成组件每次运行都在第 32 至 48 轮之间从约 25.5 MiB 升至约 33.8 MiB，随后到第 128 轮维持在附近。进程级快照只能观察总量，内存归因还需单独测量。原始样本见 [测量输出](../reports/reload)。

## 后台准备测量

```powershell
pwsh -NoProfile -File scripts/benchmark-reload.ps1 -Background -Cycles 64 -ReportDirectory reports/reload-background
```

使用相同的两份基础 bitcode，每种组件运行三个独立进程，每个进程预热 16 轮后测量 64 轮。每轮先启用两个模组，再恢复原版。等待时，宿主反复调用当前规则并核对结果，然后检查是否可以切换。

以下为归档样本的结果。

| 组件 | 发布和释放旧 JIT 的停顿 p95 | 从提交到生效 p95 |
| --- | --- | --- |
| 掉落组件，33 个目标 | 0.15–0.16 ms | 31.8–33.3 ms |
| 合成组件，98 个目标 | 0.20–0.23 ms | 272.9–279.3 ms |

两个方向合计 768 次发布。发布停顿最长约 0.63 ms，从提交到生效的计时包含后台编译和轮询等待。另有集成测试覆盖准备期间继续使用旧规则、类型或位置不匹配时保留当前 JIT，以及拒绝同时提交第二个请求。原始样本见 [后台准备测量输出](../reports/reload-background)。
