use mixin_runtime::{Api, HotReload, bundle::Manifest};
use std::borrow::Cow;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use video_game::game;

#[cfg(feature = "embedded-bitcode")]
fn game_bytes(_: &Path) -> Result<Cow<'static, [u8]>, String> {
    let game = include_bytes!("../../../artifacts/video-demo/retained_game.bc");
    let manifest = include_bytes!("../../../artifacts/video-demo/manifest.json");
    Manifest::read_embedded_game(manifest, game)?;
    Ok(Cow::Borrowed(game))
}

#[cfg(not(feature = "embedded-bitcode"))]
fn game_bytes(executable: &Path) -> Result<Cow<'static, [u8]>, String> {
    let artifacts = executable.with_file_name("artifacts").join("video-demo");
    Manifest::read_game(&artifacts)
        .map(|(_, bytes)| Cow::Owned(bytes))
        .map_err(|error| format!("{}: {error}", artifacts.display()))
}

fn play(runtime: &HotReload<'_>) -> Result<(), String> {
    runtime.call("demo::game::play", &[])?;
    Ok(())
}

fn run(mods: &[PathBuf], interactive: bool) -> Result<(), String> {
    if mods.is_empty() && !interactive {
        game::play();
        return Ok(());
    }
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let game = game_bytes(&executable)?;
    let dll = executable.with_file_name("LLVM-C.dll");
    let api = Api::load(&dll).map_err(|error| format!("{}: {error}", dll.display()))?;
    api.initialize()?;
    let std_library =
        option_env!("MIXIN_DEMO_STD_DLL").ok_or("请先用 cargo build-game 构建游戏")?;
    let std_library = executable.with_file_name(std_library);
    let libraries = [std_library.as_path()];
    // SAFETY: the demo ships the same pinned std DLL used by the host.
    // Game and mod manifests validate the rustc identity.
    let runtime = unsafe { HotReload::with_libraries(&api, &game, mods, &libraries) }
        .map_err(|error| error.to_string())?;
    if interactive {
        let mod_directory = executable.with_file_name("mods");
        return run_interactive(runtime, &mod_directory);
    }
    play(&runtime)
}

fn run_interactive(mut runtime: HotReload<'_>, mod_directory: &Path) -> Result<(), String> {
    play(&runtime)?;
    const HELP: &str = "输入 加载 <模组名或路径> 来切换模组，输入 清空 恢复原版，输入 退出 结束";
    println!("{HELP}");
    io::stdout().flush().map_err(|error| error.to_string())?;
    for line in io::stdin().lock().lines() {
        let line = line.map_err(|error| error.to_string())?;
        let trimmed = line.trim();
        let (command, argument) = trimmed.split_once(' ').unwrap_or((trimmed, ""));
        match command {
            "步骤" if argument.is_empty() => play(&runtime)?,
            "加载" if !argument.trim().is_empty() => {
                let path = PathBuf::from(argument.trim());
                let path = if path.file_name() == Some(path.as_os_str()) {
                    let installed = mod_directory.join(&path);
                    if installed.is_dir() { installed } else { path }
                } else {
                    path
                };
                match runtime.replace(&[path]) {
                    Ok(()) => {
                        println!("模组已加载");
                        play(&runtime)?;
                    }
                    Err(error) => eprintln!("加载失败：{error}"),
                }
            }
            "清空" if argument.is_empty() => {
                runtime.replace(&[]).map_err(|error| error.to_string())?;
                println!("模组已清空");
                play(&runtime)?;
            }
            "退出" if argument.is_empty() => break,
            "" => continue,
            _ => eprintln!("{HELP}"),
        }
        io::stdout().flush().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    // Abort before a panic can unwind through JIT frames.
    std::panic::set_hook(Box::new(|info| {
        eprintln!("{info}");
        std::process::abort();
    }));

    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args == ["--game-hash"] {
        let hash = std::env::current_exe()
            .map_err(|error| error.to_string())
            .and_then(|exe| game_bytes(&exe))
            .map(|bytes| mixin_runtime::bundle::sha256(&bytes));
        return match hash {
            Ok(hash) => {
                println!("{hash}");
                std::process::ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("错误：{error}");
                std::process::ExitCode::FAILURE
            }
        };
    }
    let mut args = args.into_iter();
    let mut mods = Vec::new();
    let mut interactive_mode = false;
    while let Some(option) = args.next() {
        if option == "--mod" {
            let Some(path) = args.next() else {
                eprintln!("错误：--mod 后面需要模组目录");
                return std::process::ExitCode::FAILURE;
            };
            mods.push(PathBuf::from(path));
        } else if option == "--interactive" {
            interactive_mode = true;
        } else if option == "--help" || option == "-h" {
            println!("用法：video-game.exe [--mod 模组目录]... [--interactive]");
            return std::process::ExitCode::SUCCESS;
        } else {
            eprintln!("错误：未知参数 {}", option.to_string_lossy());
            return std::process::ExitCode::FAILURE;
        }
    }

    match run(&mods, interactive_mode) {
        Ok(()) => {
            if !interactive_mode {
                println!("游戏运行完成");
                for path in mods {
                    println!("模组：已加载 {}", path.display());
                }
            }
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("错误：{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
