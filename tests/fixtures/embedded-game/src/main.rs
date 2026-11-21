//! Example host with configurable game bitcode and LLVM packaging.
#[cfg(feature = "embedded-llvm")]
mod embedded_llvm;
mod plugins;

use mixin_runtime::{
    Api, Diagnostic, Jit, Result,
    bundle::{Manifest, ModManifest},
};
use std::borrow::Cow;
use std::path::{Path, PathBuf};

fn game_bytes(_executable: &Path) -> Result<Cow<'static, [u8]>> {
    #[cfg(feature = "embedded-bitcode")]
    {
        let game = include_bytes!("../../../../artifacts/retained_game.bc");
        let manifest = include_bytes!("../../../../artifacts/manifest.json");
        Manifest::read_embedded_game(manifest, game)?;
        Ok(Cow::Borrowed(game))
    }
    #[cfg(not(feature = "embedded-bitcode"))]
    {
        let root = _executable.with_file_name("artifacts");
        let (_, game) =
            Manifest::read_game(&root).map_err(|error| format!("{}: {error}", root.display()))?;
        Ok(Cow::Owned(game))
    }
}

fn llvm_path(_executable: &Path) -> Result<PathBuf> {
    #[cfg(feature = "embedded-llvm")]
    {
        embedded_llvm::install()
    }
    #[cfg(not(feature = "embedded-llvm"))]
    {
        Ok(_executable.with_file_name("LLVM-C.dll"))
    }
}

fn patched_damage(mods: &[PathBuf], amount: f64, armor: f64) -> Result<f64> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let game = game_bytes(&executable)?;
    let codes = mods
        .iter()
        .map(|path| {
            ModManifest::read(path, &game).map_err(|error| format!("{}: {error}", path.display()))
        })
        .collect::<Result<Vec<_>>>()?;
    let modules: Vec<_> = codes.iter().map(Vec::as_slice).collect();

    let dll = llvm_path(&executable)?;
    let api = Api::load(&dll).map_err(|error| format!("{}: {error}", dll.display()))?;
    api.initialize()?;

    let (jit, _, _) = Jit::compile_many_detailed(&api, &game, &modules, true, false).map_err(
        |error: Diagnostic| match error.mod_index.and_then(|index| mods.get(index)) {
            Some(path) => format!("{}: {error}", path.display()),
            None => error.to_string(),
        },
    )?;
    jit.call_two("demo::combat::apply_damage", amount, armor)
}

fn run(mods: &[PathBuf]) -> Result<()> {
    let mut events = plugins::DamageEvents::default();
    let mut mixins = Vec::new();
    for dir in mods {
        let plugin = dir.join("plugin.dll");
        let present = |path: &Path| {
            path.try_exists()
                .map_err(|error| format!("{}: {error}", path.display()))
        };
        let has_plugin = present(&plugin)?;
        let has_mixin = present(&dir.join("mod.json"))? || present(&dir.join("code.bc"))?;
        if has_mixin {
            mixins.push(dir.clone());
        }
        if has_plugin {
            events.load(&plugin)?;
        } else if !has_mixin {
            return Err(format!(
                "{}: expected plugin.dll or a Mixin package",
                dir.display()
            ));
        }
    }
    let (amount, armor) = (20.0, 8.0);
    let damage = if mixins.is_empty() {
        llvm_fixture::combat::apply_damage(amount, armor)
    } else {
        patched_damage(&mixins, amount, armor)?
    };
    let damage = events.emit(damage);
    println!("Damage: {damage}");
    Ok(())
}

fn main() -> std::process::ExitCode {
    let mut args = std::env::args_os().skip(1);
    let mut mods = Vec::new();
    while let Some(option) = args.next() {
        if option == "--help" || option == "-h" {
            println!("game.exe [--mod DIR]...");
            #[cfg(feature = "embedded-llvm")]
            println!("game.exe --llvm-license");
            return std::process::ExitCode::SUCCESS;
        }
        #[cfg(feature = "embedded-llvm")]
        if option == "--llvm-license" {
            print!("{}", embedded_llvm::LICENSE);
            return std::process::ExitCode::SUCCESS;
        }
        if option == "--mod" {
            if let Some(path) = args.next() {
                mods.push(PathBuf::from(path));
                continue;
            }
            eprintln!("error: --mod requires a directory");
        } else {
            eprintln!("error: unknown argument {}", option.to_string_lossy());
        }
        return std::process::ExitCode::FAILURE;
    }
    match run(&mods) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
