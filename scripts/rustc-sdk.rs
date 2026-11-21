use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(status) => status,
        Err(error) => {
            eprintln!("Rust Mixin SDK: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<ExitCode> {
    let mut arguments = env::args_os().skip(1);
    let compiler = arguments
        .next()
        .ok_or_else(|| io::Error::other("missing rustc executable"))?;
    let sdk = PathBuf::from(
        env::var_os("RUST_MIXIN_COMPILED_SDK")
            .ok_or_else(|| io::Error::other("missing compiled SDK directory"))?,
    );
    let externs = fs::read_to_string(sdk.join("externs.txt"))?;
    let mut command = Command::new(compiler);
    let mut matched = false;
    while let Some(argument) = arguments.next() {
        if argument == OsStr::new("--extern") {
            let declaration = arguments
                .next()
                .ok_or_else(|| io::Error::other("missing --extern value"))?;
            if let Some((name, library)) = replacement(&declaration, &externs) {
                let mut value = OsString::from(name);
                value.push("=");
                value.push(sdk.join(library));
                command.args([OsString::from("--extern"), value]);
                matched = true;
                continue;
            }
            command.args([argument, declaration]);
        } else {
            command.arg(argument);
        }
    }
    if matched {
        let mut search_path = OsString::from("dependency=");
        search_path.push(sdk);
        command.arg("-L").arg(search_path);
    }
    let status = command.status()?;
    Ok(ExitCode::from(
        status
            .code()
            .and_then(|code| u8::try_from(code).ok())
            .unwrap_or(1),
    ))
}

fn replacement<'a>(declaration: &'a OsStr, externs: &'a str) -> Option<(&'a str, &'a str)> {
    let (name, _) = declaration.to_str()?.split_once('=')?;
    externs
        .lines()
        .filter_map(|line| line.split_once('='))
        .find(|(alias, _)| *alias == name)
}
