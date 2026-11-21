//! AOT/JIT validation and measurements for the combat example.
mod fixture;
mod memory;
mod reload;
use fixture::*;
use mixin_runtime::{
    Api, Jit, Result,
    bundle::{Manifest, sha256},
    llvm,
};
use serde::Serialize;
use std::{
    hint::black_box,
    path::{Path, PathBuf},
    time::Instant,
};

#[derive(Serialize)]
struct Samples {
    ns_per_operation: Vec<f64>,
    median_ns: f64,
    min_ns: f64,
    max_ns: f64,
}

fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted[sorted.len() / 2]
}

impl Samples {
    fn from(values: Vec<f64>) -> Self {
        Self {
            median_ns: median(&values),
            min_ns: values.iter().copied().fold(f64::INFINITY, f64::min),
            max_ns: values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            ns_per_operation: values,
        }
    }
}

#[derive(Serialize)]
struct Comparison {
    name: String,
    iterations: u64,
    samples: usize,
    aot: Samples,
    jit: Samples,
    paired_ratios: Vec<f64>,
    median_paired_ratio: f64,
    meaning: String,
}

fn external_loop(function: DamageFn, n: u64, seed: f64) -> f64 {
    let function = black_box(function);
    let mut sum = 0.0;
    for i in 0..n {
        let amount = seed + (i & 31) as f64 * 0.125;
        let armor = (i & 7) as f64;
        sum += function(amount, armor);
    }
    black_box(sum)
}

fn measure(mut f: impl FnMut() -> f64) -> (f64, f64) {
    let start = Instant::now();
    let result = black_box(f());
    (start.elapsed().as_secs_f64() * 1e9, result)
}

fn compare(
    name: &str,
    n: u64,
    samples: usize,
    mut aot: impl FnMut() -> f64,
    mut jit: impl FnMut() -> f64,
    meaning: &str,
) -> Result<Comparison> {
    for _ in 0..3 {
        black_box(aot());
        black_box(jit());
    }
    let (mut aot_samples, mut jit_samples, mut ratios) = (Vec::new(), Vec::new(), Vec::new());
    for i in 0..samples {
        // Alternate order to reduce systematic warmup/thermal bias.
        let (a, j) = if i % 2 == 0 {
            let a = measure(&mut aot);
            let j = measure(&mut jit);
            (a, j)
        } else {
            let j = measure(&mut jit);
            let a = measure(&mut aot);
            (a, j)
        };
        same_float(a.1, j.1, name)?;
        aot_samples.push(a.0 / n as f64);
        jit_samples.push(j.0 / n as f64);
        ratios.push(j.0 / a.0);
    }
    let ratio = median(&ratios);
    let comparison = Comparison {
        name: name.into(),
        iterations: n,
        samples,
        aot: Samples::from(aot_samples),
        jit: Samples::from(jit_samples),
        paired_ratios: ratios,
        median_paired_ratio: ratio,
        meaning: meaning.into(),
    };
    println!(
        "{name}: AOT {:.3} ns, JIT {:.3} ns, paired median {:.3}x",
        comparison.aot.median_ns, comparison.jit.median_ns, comparison.median_paired_ratio
    );
    Ok(comparison)
}

#[derive(Serialize)]
struct Report {
    schema: u32,
    manifest: Manifest,
    llvm_runtime: [u32; 3],
    dll_bytes: u64,
    dll_sha256: String,
    runner_bytes: u64,
    bitcode_bytes: usize,
    dll_load_initialize_ms: f64,
    empty_path: bool,
    checks: usize,
    base_value: f64,
    patched_value: f64,
    base_load_trials: Vec<llvm::Timing>,
    patched_load_trials: Vec<llvm::Timing>,
    base_load_median_ms: f64,
    patched_load_median_ms: f64,
    memory_before_llvm: memory::Snapshot,
    memory_after_llvm: memory::Snapshot,
    memory_with_two_jits: memory::Snapshot,
    memory_after_benchmarks: memory::Snapshot,
    memory_after_jit_disposal: memory::Snapshot,
    comparisons: Vec<Comparison>,
    limits: Vec<String>,
}

struct Args {
    dll: PathBuf,
    artifacts: PathBuf,
    report: PathBuf,
    iterations: u64,
    samples: usize,
    load_trials: usize,
    check_only: bool,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut args = Self {
            dll: ".tools/llvm-21.1.3/bin/LLVM-C.dll".into(),
            artifacts: "artifacts".into(),
            report: "reports/latest.json".into(),
            iterations: 8_000_000,
            samples: 15,
            load_trials: 7,
            check_only: false,
        };
        let mut rest = std::env::args().skip(1);
        while let Some(key) = rest.next() {
            if key == "--check-only" {
                args.check_only = true;
                continue;
            }
            let value = rest
                .next()
                .ok_or_else(|| format!("Missing value for {key}"))?;
            match key.as_str() {
                "--dll" => args.dll = value.into(),
                "--artifacts" => args.artifacts = value.into(),
                "--report" => args.report = value.into(),
                "--iterations" => {
                    args.iterations = value.parse().map_err(|_| "Invalid iterations")?
                }
                "--samples" => args.samples = value.parse().map_err(|_| "Invalid samples")?,
                "--load-trials" => {
                    args.load_trials = value.parse().map_err(|_| "Invalid load trials")?
                }
                _ => return Err(format!("Unknown argument {key}")),
            }
        }
        if args.iterations == 0 || args.samples < 3 || args.load_trials < 1 {
            return Err("Require iterations > 0, samples >= 3, load-trials >= 1".into());
        }
        Ok(args)
    }
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn run() -> Result<()> {
    let args = Args::parse()?;
    let (manifest, game, handler) = read_fixture(&args.artifacts)?;
    let dll = args.dll.canonicalize().map_err(|e| e.to_string())?;
    let memory_before_llvm = memory::snapshot()?;
    let load_start = Instant::now();
    let api = Api::load(&dll)?;
    let llvm_runtime = api.initialize()?;
    let dll_load_initialize_ms = load_start.elapsed().as_secs_f64() * 1000.0;
    let memory_after_llvm = memory::snapshot()?;
    let (base, _, base_evidence) = Jit::compile(&api, &game, &handler, false, true)?;
    let (patched, _, patched_evidence) = Jit::compile(&api, &game, &handler, true, true)?;
    let base = Demo::new(base)?;
    let patched = Demo::new(patched)?;
    let checks = verify(&base, &patched)?;
    let memory_with_two_jits = memory::snapshot()?;
    println!(
        "Verified {checks} comparisons; damage: {} -> {}; unmodified callee: {}",
        (base.damage)(20.0, 8.0),
        (patched.damage)(20.0, 8.0),
        (patched.armor)(20.0, 8.0)
    );
    write_json(
        &args.artifacts.join("call-index.json"),
        &base_evidence.index,
    )?;
    write_json(
        &args.artifacts.join("logical-index.json"),
        &base_evidence.logical_index,
    )?;
    write_json(
        &args.artifacts.join("patch-plan.json"),
        &patched_evidence.plan,
    )?;
    for (name, evidence) in [("base", base_evidence), ("patched", patched_evidence)] {
        std::fs::write(
            args.artifacts
                .join(format!("{name}.before-optimization.ll")),
            evidence.before.unwrap(),
        )
        .map_err(|e| e.to_string())?;
        std::fs::write(
            args.artifacts.join(format!("{name}.optimized.ll")),
            evidence.after.unwrap(),
        )
        .map_err(|e| e.to_string())?;
    }
    let (mut base_load_trials, mut patched_load_trials) = (Vec::new(), Vec::new());
    for _ in 0..args.load_trials {
        let (_, timing, _) = Jit::compile(&api, &game, &handler, false, false)?;
        base_load_trials.push(timing);
        let (_, timing, _) = Jit::compile(&api, &game, &handler, true, false)?;
        patched_load_trials.push(timing);
    }
    let base_load_median_ms = median(
        &base_load_trials
            .iter()
            .map(|t| t.total_ms)
            .collect::<Vec<_>>(),
    );
    let patched_load_median_ms = median(
        &patched_load_trials
            .iter()
            .map(|t| t.total_ms)
            .collect::<Vec<_>>(),
    );
    println!(
        "Warm process build/materialize median: base {base_load_median_ms:.3} ms, patched {patched_load_median_ms:.3} ms"
    );
    let mut comparisons = Vec::new();
    if !args.check_only {
        let n = args.iterations;
        let samples = args.samples;
        comparisons.push(compare("base_external_calls", n, samples,
            || external_loop(black_box(llvm_fixture::fixture_damage as DamageFn), n, black_box(20.0)),
            || external_loop(base.damage, n, black_box(20.0)),
            "One indirect native boundary per operation; both AOT and JIT use black-boxed function pointers.")?);
        comparisons.push(compare(
            "base_internal_batch",
            n,
            samples,
            || black_box(llvm_fixture::fixture_batch as BatchFn)(black_box(n), black_box(20.0)),
            || black_box(base.batch)(black_box(n), black_box(20.0)),
            "One boundary for the entire loop; optimizer can inline internal component calls.",
        )?);
        comparisons.push(compare("patched_external_calls", n, samples,
            || external_loop(black_box(aot_patched_damage as DamageFn), n, black_box(20.0)),
            || external_loop(patched.damage, n, black_box(20.0)),
            "Identical adjusted arithmetic in AOT and transformed JIT; not compared to cheaper unmodified logic.")?);
        comparisons.push(compare("patched_internal_batch", n, samples,
            || black_box(aot_patched_batch as BatchFn)(black_box(n), black_box(20.0)),
            || black_box(patched.batch)(black_box(n), black_box(20.0)),
            "Identical adjusted loop in AOT and transformed JIT; final LLVM optimization may inline handler.")?);
    }
    let memory_after_benchmarks = memory::snapshot()?;
    let base_value = (base.damage)(20.0, 8.0);
    let patched_value = (patched.damage)(20.0, 8.0);
    drop(patched);
    drop(base);
    let memory_after_jit_disposal = memory::snapshot()?;
    let dll_data = std::fs::read(&dll).map_err(|e| e.to_string())?;
    let report = Report { schema: 1, manifest, llvm_runtime,
        dll_bytes: dll_data.len() as u64, dll_sha256: sha256(&dll_data),
        runner_bytes: std::fs::metadata(std::env::current_exe().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?.len(), bitcode_bytes: game.len() + handler.len(),
        dll_load_initialize_ms, empty_path: std::env::var_os("PATH").is_none_or(|p| p.is_empty()),
        checks, base_value, patched_value,
        base_load_trials, patched_load_trials, base_load_median_ms, patched_load_median_ms, comparisons,
        memory_before_llvm, memory_after_llvm, memory_with_two_jits,
        memory_after_benchmarks, memory_after_jit_disposal,
        limits: vec![
            "Trusted no_std fixture/mod, scalar C boundaries, one OS/CPU, no heap/panic/unwinding/shared state.".into(),
            "Logical IDs and symbol mappings come from the experimental attribute SDK; only 0..=4 f64 target boundaries are indexed, not arbitrary Rust signatures.".into(),
            "Calls are synchronous; this experiment has no runtime publication/hot-update protocol.".into(),
            "DLL timing is this process's first load, not a reboot-cold disk benchmark; load trials reuse warm process/caches.".into(),
            "AOT uses rustc O3/thin LTO. Retained bitcode bypasses LLVM passes, then LLVM C API runs default<O3>. Pipelines are not identical.".into(),
            "No performance claim for full games, arbitrary Rust, reference capture, multiplayer or multithreaded workloads.".into(),
        ],
    };
    write_json(&args.report, &report)?;
    println!("Report: {}", args.report.display());
    Ok(())
}

fn main() {
    let result = if std::env::args_os()
        .nth(1)
        .is_some_and(|arg| arg == "reload")
    {
        reload::run(std::env::args_os().skip(2))
    } else {
        run()
    };
    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
