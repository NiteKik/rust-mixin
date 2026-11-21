//! Load independently built Rust Mixin packages through the runtime library.
use mixin_runtime::{
    Api, Diagnostic, Jit, ScalarType, ScalarValue, Stage,
    bundle::{Manifest, ModManifest, sha256},
};
use serde::Serialize;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, String>;

#[derive(Serialize)]
struct CliError {
    #[serde(flatten)]
    diagnostic: Diagnostic,
    #[serde(skip_serializing_if = "Option::is_none")]
    mod_path: Option<PathBuf>,
}

impl CliError {
    fn new(stage: Stage, error: impl Into<String>) -> Self {
        Self {
            diagnostic: Diagnostic::new(stage, error),
            mod_path: None,
        }
    }

    fn from_runtime(diagnostic: Diagnostic, mods: &[PathBuf]) -> Self {
        let mod_path = diagnostic
            .mod_index
            .and_then(|index| mods.get(index))
            .cloned();
        Self {
            diagnostic,
            mod_path,
        }
    }
}

struct Args {
    dll: PathBuf,
    artifacts: PathBuf,
    report: PathBuf,
    mods: Vec<PathBuf>,
    call: String,
    arg0: f64,
    arg1: f64,
    legacy_args: bool,
    scalar_args: Vec<String>,
    plan: bool,
}
impl Args {
    fn parse() -> Result<Self> {
        let mut args = Self {
            dll: ".tools/llvm-21.1.3/bin/LLVM-C.dll".into(),
            artifacts: "artifacts".into(),
            report: "reports/latest.json".into(),
            mods: Vec::new(),
            call: String::new(),
            arg0: 20.0,
            arg1: 8.0,
            legacy_args: false,
            scalar_args: Vec::new(),
            plan: false,
        };
        let mut rest = std::env::args().skip(1).peekable();
        if rest
            .peek()
            .is_some_and(|argument| argument == "plan" || argument == "--plan")
        {
            rest.next();
            args.plan = true;
            args.report = "reports/plan.json".into();
        }
        while let Some(key) = rest.next() {
            let value = rest
                .next()
                .ok_or_else(|| format!("Missing value for {key}"))?;
            match key.as_str() {
                "--dll" => args.dll = value.into(),
                "--artifacts" => args.artifacts = value.into(),
                "--report" => args.report = value.into(),
                "--mod" => args.mods.push(value.into()),
                "--call" if !args.plan => args.call = value,
                "--arg0" if !args.plan => {
                    args.arg0 = value.parse().map_err(|_| "Invalid arg0")?;
                    args.legacy_args = true;
                }
                "--arg1" if !args.plan => {
                    args.arg1 = value.parse().map_err(|_| "Invalid arg1")?;
                    args.legacy_args = true;
                }
                "--arg" if !args.plan => args.scalar_args.push(value),
                _ => return Err(format!("Unknown argument {key}")),
            }
        }
        if args.call.is_empty() && !args.plan {
            return Err("Require --call <logical target ID>".into());
        }
        if args.legacy_args && !args.scalar_args.is_empty() {
            return Err("Use either --arg or --arg0/--arg1".into());
        }
        Ok(args)
    }

    fn call_args(&self, parameters: &[ScalarType]) -> Result<Vec<ScalarValue>> {
        if self.scalar_args.is_empty() {
            if parameters == [ScalarType::F64; 2] {
                return Ok(vec![
                    ScalarValue::F64(self.arg0),
                    ScalarValue::F64(self.arg1),
                ]);
            }
            if self.legacy_args {
                return Err("--arg0/--arg1 require two f64 parameters".into());
            }
        }
        if self.scalar_args.len() != parameters.len() {
            return Err(format!(
                "expected {} --arg values, got {}",
                parameters.len(),
                self.scalar_args.len()
            ));
        }
        parameters
            .iter()
            .zip(&self.scalar_args)
            .enumerate()
            .map(|(index, (&kind, text))| {
                ScalarValue::parse(kind, text).map_err(|error| format!("argument {index}: {error}"))
            })
            .collect()
    }
}
fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn run(args: &Args) -> std::result::Result<(), CliError> {
    let (manifest, game) = Manifest::read_game(&args.artifacts)
        .map_err(|error| CliError::new(Stage::Manifest, error))?;
    let codes: Vec<_> = args
        .mods
        .iter()
        .map(|path| {
            ModManifest::read(path, &game).map_err(|error| CliError {
                diagnostic: Diagnostic::new(Stage::ModManifest, error),
                mod_path: Some(path.clone()),
            })
        })
        .collect::<std::result::Result<_, _>>()?;
    let modules: Vec<&[u8]> = codes.iter().map(Vec::as_slice).collect();
    let dll = args
        .dll
        .canonicalize()
        .map_err(|error| CliError::new(Stage::LoadLlvm, error.to_string()))?;
    let api = Api::load(&dll).map_err(|error| CliError::new(Stage::LoadLlvm, error))?;
    api.initialize()
        .map_err(|error| CliError::new(Stage::InitializeLlvm, error))?;
    let runtime_error = |error| CliError::from_runtime(error, &args.mods);
    if args.plan {
        let preflight = Jit::plan_many(&api, &game, &modules).map_err(runtime_error)?;
        println!("Validated {} patch(es).", preflight.patches.len());
        for patch in &preflight.patches {
            println!(
                "{}: {} -> {} ordinal {} / {}",
                patch.id, patch.target, patch.callee, patch.ordinal, patch.matches
            );
        }
        write_json(
            &args.report,
            &serde_json::json!({
                "schema": 3,
                "mode": "plan",
                "manifest": manifest,
                "game_sha256": sha256(&game),
                "mods": args.mods,
                "index": preflight.index,
                "plan": preflight.patches,
                "empty_path": std::env::var_os("PATH").is_none_or(|p| p.is_empty())
            }),
        )
        .map_err(|error| CliError::new(Stage::WriteReport, error))?;
        return Ok(());
    }
    let (patched, timing, evidence) =
        Jit::compile_many_detailed(&api, &game, &modules, true, true).map_err(runtime_error)?;
    let (base, _, _) =
        Jit::compile_many_detailed(&api, &game, &[], false, false).map_err(runtime_error)?;
    let call_error = |error| CliError::new(Stage::Call, format!("{}: {error}", args.call));
    let target = base.index().target(&args.call).map_err(call_error)?;
    let values = args
        .call_args(target.parameter_types())
        .map_err(call_error)?;
    let base_value = base.call(&args.call, &values).map_err(call_error)?;
    let patched_value = patched.call(&args.call, &values).map_err(call_error)?;
    let base_unchanged = base.call(&args.call, &values).map_err(call_error)?;
    let arguments = values.iter().map(ToString::to_string).collect::<Vec<_>>();
    println!("Call: {}({})", args.call, arguments.join(", "));
    println!("Base: {base_value}");
    println!("Patched: {patched_value}");
    println!("Independent base still unchanged: {base_unchanged}");
    for patch in &evidence.plan {
        println!(
            "Applied {}: {} -> {} ordinal {} / {}",
            patch.id, patch.target, patch.callee, patch.ordinal, patch.matches
        );
    }
    write_json(
        &args.report,
        &serde_json::json!({
            "schema":2,"mode":"external_mods","manifest":manifest,"game_sha256":sha256(&game),
            "mods":args.mods,"call":args.call,"base_value":base_value,"patched_value":patched_value,
            "base_unchanged":base_unchanged,"timing":timing,"index":evidence.logical_index,"plan":evidence.plan,
            "empty_path":std::env::var_os("PATH").is_none_or(|p|p.is_empty())
        }),
    )
    .map_err(|error| CliError::new(Stage::WriteReport, error))?;
    if let Some(ir) = evidence.before {
        std::fs::write(args.report.with_extension("before.ll"), ir)
            .map_err(|error| CliError::new(Stage::WriteEvidence, error.to_string()))?;
    }
    if let Some(ir) = evidence.after {
        std::fs::write(args.report.with_extension("optimized.ll"), ir)
            .map_err(|error| CliError::new(Stage::WriteEvidence, error.to_string()))?;
    }
    Ok(())
}

fn main() {
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        println!(
            "mixin-cli --call <logical ID> [--mod DIR]... [--arg VALUE]... [--dll FILE --artifacts DIR --report FILE]\n\
             mixin-cli plan [--mod DIR]... [--dll FILE --artifacts DIR --report FILE]"
        );
        return;
    }
    let parsed = Args::parse();
    match parsed {
        Ok(args) => {
            if let Err(error) = run(&args) {
                eprintln!("error: {}", error.diagnostic);
                if let Some(path) = &error.mod_path {
                    eprintln!("Mod: {}", path.display());
                }
                if let Err(report_error) = write_json(
                    &args.report,
                    &serde_json::json!({"schema": 3, "mode": "error", "diagnostic": error}),
                ) {
                    eprintln!(
                        "Could not write report {}: {report_error}",
                        args.report.display()
                    );
                }
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("error: arguments: {error}");
            std::process::exit(1);
        }
    }
}
