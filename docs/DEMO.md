# 录屏演示

游戏包含三个动作。原版流程是打开冰箱门、把大象放进冰箱、关上冰箱门。

```rust
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
```

游戏和模组都用标准库的 `println!`。构建脚本会将当前固定工具链的 Rust 标准库 DLL 放在游戏旁。游戏动态链接这份标准库，LLVM 也从同一个 DLL 解析热加载代码所需的输出函数。

```rust
// 在第一步之前输出
#[mixin_sdk::inject_head("demo::game::play")]
fn before_play() {
    println!("我在这里");
}

// 在开门调用结束后输出
#[mixin_sdk::inject_after_call("demo::game::play", callee = "demo::game::open_fridge")]
fn inside_play() {
    println!("我在这里");
}

// 在最后一步之后输出
#[mixin_sdk::inject_return("demo::game::play")]
fn after_play() {
    println!("我在这里");
}
```

目标函数可以直接写成第一个字符串参数。省略 `id` 时，SDK 用 crate 名和回调函数名生成 ID。中间模组默认要求恰好匹配一次开门调用。

## 一键演示

源码构建使用 Windows x64 MSVC 和 rustc 1.92.0。下文的 Cargo 构建命令还需要 PowerShell 7（`pwsh`）。首次在项目根目录准备 LLVM 21.1.3。准备脚本使用 7-Zip，安装位置不同可传入 `-SevenZip`。

```powershell
pwsh -NoProfile -File .\scripts\prepare-llvm.ps1
.\demo.cmd
```

脚本按录屏顺序执行四步。

1. 编译游戏并导出游戏代码。
2. 启动原版游戏，输出三步冰箱操作。
3. 分别编译三个模组。
4. 每次运行加载一个模组，分别在开门前、开门后和关门后输出“我在这里”。

每次加载后都会打印模组目录。用 `pwsh -NoProfile -File scripts/package.ps1` 打包后，可运行 `.\dist\demo.cmd` 直接使用已编译的游戏和模组。

## 同一进程动态加载

两个 PowerShell 窗口都进入项目根目录。在第一个窗口编译游戏，然后启动交互模式。

```powershell
cargo build-game --output ".\录屏游戏"
.\录屏游戏\video-game.exe --interactive
```

根目录 `Cargo.toml` 的 `[workspace.metadata.video-demo]` 配置默认输出目录和 bitcode 模式。默认将 bitcode 和校验清单一起嵌入 EXE。构建时可用 `--bitcode external` 将两份文件放到输出目录的 `artifacts\video-demo` 下，也可用 `--bitcode embedded` 显式选择内嵌。模组构建命令会从对应游戏读取并校验 bitcode。

游戏启动时会自动打印原版三步流程。保持游戏运行，在第二个窗口按需编译模组。

```powershell
cargo build-mod-before --output ".\录屏游戏"
cargo build-mod-inside --output ".\录屏游戏"
cargo build-mod-after --output ".\录屏游戏"
```

四条 Cargo 命令使用同一个输出目录。省略 `--output` 时读取 `Cargo.toml` 里的 `output-directory`，初始值为 `target\video-demo`。每个模组会放到该目录下的 `mods\<模组名>`。回到游戏窗口，逐条输入加载命令，每次加载成功后游戏会立即打印新效果。

```text
加载 video-before
加载 video-inside
加载 video-after
```

每次成功加载会替换当前模组。“我在这里”分别出现在开门前、开门后和关门后。手动复制模组时，将包含 `mod.json` 和 `code.bc` 的整个模组目录放进游戏旁的 `mods`，再输入 `加载 <模组名>`。输入 `清空` 恢复原版，输入 `退出` 结束游戏。

## 手动录屏节奏

先展示 `examples/video-game/src/lib.rs`，再展示 `examples/mods/` 下的 `video-before`、`video-inside` 和 `video-after`，每个模组的代码都在 `src/lib.rs`。
