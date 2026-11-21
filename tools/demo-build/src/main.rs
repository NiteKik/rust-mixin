use std::env;
use std::path::{Path, absolute};
use std::process::{self, Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<_> = env::args().skip(1).collect();
    let (mut script_args, options) = match args.as_slice() {
        [action, options @ ..] if action == "game" => (vec!["-GameOnly"], options),
        [action, name, options @ ..] if action == "mod" => (vec!["-Mod", name.as_str()], options),
        _ => return usage(),
    };

    let mut output = None;
    let mut bitcode = None;
    for option in options.chunks(2) {
        match option {
            [flag, directory]
                if flag == "--output" && !directory.is_empty() && output.is_none() =>
            {
                match absolute(directory) {
                    Ok(path) => output = Some(path),
                    Err(error) => {
                        eprintln!("无法解析输出目录：{error}");
                        return ExitCode::FAILURE;
                    }
                }
            }
            [flag, mode] if flag == "--bitcode" && args[0] == "game" && bitcode.is_none() => {
                bitcode = Some(match mode.as_str() {
                    "embedded" => "Embedded",
                    "external" => "External",
                    _ => return usage(),
                });
            }
            _ => return usage(),
        }
    }
    if let Some(mode) = bitcode {
        script_args.extend(["-Bitcode", mode]);
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut command = Command::new("pwsh");
    command
        .current_dir(root)
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            "scripts/video-demo.ps1",
        ])
        .args(script_args);
    if let Some(directory) = output {
        command.arg("-OutputDirectory").arg(directory);
    }
    match command.status() {
        Ok(status) => process::exit(status.code().unwrap_or(1)),
        Err(error) => {
            eprintln!("无法启动 PowerShell：{error}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> ExitCode {
    eprintln!("用法：cargo build-game [--output 目录] [--bitcode embedded|external]");
    eprintln!("      cargo build-mod <模组名> [--output 目录]");
    ExitCode::FAILURE
}
