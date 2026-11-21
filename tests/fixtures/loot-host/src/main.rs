use mixin_runtime::{Api, BackgroundReload, HotReload, Result, ScalarValue, bundle::Manifest};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const DROP_COUNT: &str = "demo::loot::drop_count";
const DROP_ARGS: [ScalarValue; 4] = [
    ScalarValue::I32(2),
    ScalarValue::Bool(false),
    ScalarValue::F32(1.0),
    ScalarValue::U64(20),
];

fn run(directory: &Path, mods: &[PathBuf], plan: bool, interactive: bool) -> Result<()> {
    let artifacts = directory.join("artifacts");
    let (_, game) = Manifest::read_game(&artifacts)?;
    let api = Api::load(&directory.join("LLVM-C.dll"))?;
    api.initialize()?;
    if plan {
        let plan = HotReload::plan(&api, &game, mods).map_err(|error| error.to_string())?;
        for patch in plan.patches {
            println!("{} -> {} ({} match)", patch.id, patch.target, patch.matches);
        }
        return Ok(());
    }
    let runtime = HotReload::new(&api, &game, mods).map_err(|error| error.to_string())?;
    if interactive {
        return std::thread::scope(|scope| run_interactive(runtime.background(scope)));
    }
    println!("Loot: {}", runtime.call(DROP_COUNT, &DROP_ARGS)?);
    Ok(())
}

fn drop_count(value: ScalarValue) -> Result<i32> {
    match value {
        ScalarValue::I32(count) => Ok(count),
        value => Err(format!("expected an integer drop count, got {value}")),
    }
}

fn run_interactive(mut runtime: BackgroundReload<'_, '_>) -> Result<()> {
    let mut total = 0_i64;
    let mut pending_count = 0;
    println!("Commands: drop, mods [DIR|DIR...], prepare [DIR|DIR...], activate, wait, quit");
    io::stdout().flush().map_err(|error| error.to_string())?;
    for line in io::stdin().lock().lines() {
        let line = line.map_err(|error| error.to_string())?;
        let line = line.trim();
        let (command, arguments) = line.split_once(' ').unwrap_or((line, ""));
        match command {
            "drop" if arguments.is_empty() => {
                let count = drop_count(runtime.call(DROP_COUNT, &DROP_ARGS)?)?;
                total += i64::from(count);
                println!("Loot: {count}, total: {total}");
            }
            "mods" | "prepare" => {
                let paths = if arguments.is_empty() {
                    Vec::new()
                } else {
                    arguments
                        .split('|')
                        .map(|path| PathBuf::from(path.trim()))
                        .collect()
                };
                let count = paths.len();
                match runtime.request(paths) {
                    Ok(()) => {
                        pending_count = count;
                        if command == "mods" {
                            report_activation(runtime.finish(), pending_count);
                        } else {
                            println!("Preparing mods: {count}");
                        }
                    }
                    Err(error) => eprintln!("error: {error}"),
                }
            }
            "activate" if arguments.is_empty() => {
                report_activation(runtime.try_activate(), pending_count)
            }
            "wait" if arguments.is_empty() => report_activation(runtime.finish(), pending_count),
            "quit" if arguments.is_empty() => break,
            "" => continue,
            _ => eprintln!("error: unknown command {command}"),
        }
        io::stdout().flush().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn report_activation(result: mixin_runtime::DiagnosticResult<bool>, count: usize) {
    match result {
        Ok(true) => println!("Mods applied: {count}"),
        Ok(false) => println!("No update ready"),
        Err(error) => eprintln!("error: {error}"),
    }
}

fn command() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let mut mods = Vec::new();
    let mut plan = false;
    let mut interactive = false;
    while let Some(arg) = args.next() {
        if arg == "--mod" {
            mods.push(PathBuf::from(
                args.next().ok_or("--mod requires a directory")?,
            ));
        } else if arg == "--plan" {
            plan = true;
        } else if arg == "--interactive" {
            interactive = true;
        } else if arg == "--help" || arg == "-h" {
            println!("loot-game [--plan | --interactive] [--mod DIR]...");
            return Ok(());
        } else {
            return Err(format!("unknown argument {}", arg.to_string_lossy()));
        }
    }
    if plan && interactive {
        return Err("--plan and --interactive cannot be combined".into());
    }
    let game = std::env::current_exe().map_err(|error| error.to_string())?;
    run(
        game.parent().ok_or("missing executable directory")?,
        &mods,
        plan,
        interactive,
    )
}

fn main() -> ExitCode {
    match command() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
