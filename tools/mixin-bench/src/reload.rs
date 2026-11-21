use crate::{memory, write_json};
use mixin_runtime::{
    Api, BackgroundReload, HotReload, Result, ScalarValue,
    bundle::{Manifest, sha256},
};
use serde::Serialize;
use std::{
    ffi::OsString,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

const TARGET: &str = "demo::loot::drop_count";
const ARGS: [ScalarValue; 4] = [
    ScalarValue::I32(2),
    ScalarValue::Bool(false),
    ScalarValue::F32(1.0),
    ScalarValue::U64(20),
];
const WARMUP_CYCLES: usize = 16;
const FRAME_MS: f64 = 1000.0 / 60.0;

struct Options {
    dll: PathBuf,
    artifacts: PathBuf,
    report: PathBuf,
    mods: Vec<PathBuf>,
    cycles: usize,
    background: bool,
}

impl Options {
    fn parse(mut args: impl Iterator<Item = OsString>) -> Result<Self> {
        let mut options = Self {
            dll: ".tools/llvm-21.1.3/bin/LLVM-C.dll".into(),
            artifacts: "artifacts/scalar".into(),
            report: "reports/reload/latest.json".into(),
            mods: Vec::new(),
            cycles: 128,
            background: false,
        };
        while let Some(key) = args.next() {
            if key == "--background" {
                options.background = true;
                continue;
            }
            let value = args
                .next()
                .ok_or_else(|| format!("Missing value for {key:?}"))?;
            match key.to_str() {
                Some("--dll") => options.dll = value.into(),
                Some("--artifacts") => options.artifacts = value.into(),
                Some("--report") => options.report = value.into(),
                Some("--mod") => options.mods.push(value.into()),
                Some("--cycles") => {
                    options.cycles = value
                        .to_str()
                        .and_then(|value| value.parse().ok())
                        .ok_or("Invalid cycle count")?;
                }
                _ => return Err(format!("Unknown reload argument {key:?}")),
            }
        }
        if options.mods.is_empty() || options.cycles == 0 {
            return Err("Reload benchmark requires a mod package and at least one cycle".into());
        }
        Ok(options)
    }
}

#[derive(Serialize)]
struct Pause {
    samples_ms: Vec<f64>,
    median_ms: f64,
    p95_ms: f64,
    max_ms: f64,
    over_60_hz_frame: usize,
}

impl Pause {
    fn from_samples(samples_ms: Vec<f64>) -> Self {
        let mut sorted = samples_ms.clone();
        sorted.sort_by(f64::total_cmp);
        let middle = sorted.len() / 2;
        Self {
            median_ms: (sorted[(sorted.len() - 1) / 2] + sorted[middle]) / 2.0,
            p95_ms: sorted[(sorted.len() * 95).div_ceil(100) - 1],
            max_ms: sorted[sorted.len() - 1],
            over_60_hz_frame: sorted.iter().filter(|&&ms| ms > FRAME_MS).count(),
            samples_ms,
        }
    }
}

#[derive(Serialize)]
struct Checkpoint {
    cycle: usize,
    process: memory::Snapshot,
}

#[derive(Serialize)]
struct Report {
    background: bool,
    game_sha256: String,
    bitcode_bytes: usize,
    indexed_targets: usize,
    mods: Vec<PathBuf>,
    warmup_cycles: usize,
    measured_cycles: usize,
    enable: Pause,
    restore_base: Pause,
    #[serde(skip_serializing_if = "Option::is_none")]
    background_timings: Option<BackgroundTimings>,
    memory_before_llvm: memory::Snapshot,
    memory_after_llvm: memory::Snapshot,
    memory_after_restore: Vec<Checkpoint>,
    memory_after_jit_disposal: memory::Snapshot,
}

#[derive(Serialize)]
struct BackgroundTimings {
    request_pause: Pause,
    time_until_activation: Pause,
    calls_on_old_generation: usize,
}

#[derive(Default)]
struct Switch {
    pause_ms: f64,
    request_ms: f64,
    elapsed_ms: f64,
    old_calls: usize,
}

fn replace(runtime: &mut HotReload<'_>, mods: &[PathBuf], expected: i32) -> Result<Switch> {
    let start = Instant::now();
    runtime.replace(mods).map_err(|error| error.to_string())?;
    let pause_ms = start.elapsed().as_secs_f64() * 1000.0;
    check_loot(runtime.call(TARGET, &ARGS)?, expected)?;
    Ok(Switch {
        pause_ms,
        ..Switch::default()
    })
}

fn replace_in_background(
    runtime: &mut BackgroundReload<'_, '_>,
    mods: &[PathBuf],
    expected: i32,
) -> Result<Switch> {
    let request = mods.to_vec();
    let start = Instant::now();
    runtime
        .request(request)
        .map_err(|error| error.to_string())?;
    let request_ms = start.elapsed().as_secs_f64() * 1000.0;
    let previous = if mods.is_empty() { 8 } else { 3 };
    let mut old_calls = 0;
    loop {
        check_loot(runtime.call(TARGET, &ARGS)?, previous)?;
        old_calls += 1;
        let start_activation = Instant::now();
        let activated = runtime.try_activate().map_err(|error| error.to_string())?;
        let pause_ms = start_activation.elapsed().as_secs_f64() * 1000.0;
        if activated {
            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
            check_loot(runtime.call(TARGET, &ARGS)?, expected)?;
            return Ok(Switch {
                pause_ms,
                request_ms,
                elapsed_ms,
                old_calls,
            });
        }
        thread::sleep(Duration::from_millis(1));
    }
}

fn check_loot(actual: ScalarValue, expected: i32) -> Result<()> {
    if actual != ScalarValue::I32(expected) {
        return Err(format!(
            "Expected {expected} loot after replacement, got {actual}"
        ));
    }
    Ok(())
}

struct Measurements {
    enable: Vec<Switch>,
    restore: Vec<Switch>,
    checkpoints: Vec<Checkpoint>,
}

fn measure(
    options: &Options,
    mut switch: impl FnMut(&[PathBuf], i32) -> Result<Switch>,
) -> Result<Measurements> {
    let mut enable = Vec::with_capacity(options.cycles);
    let mut restore = Vec::with_capacity(options.cycles);
    for _ in 0..WARMUP_CYCLES {
        switch(&options.mods, 8)?;
        switch(&[], 3)?;
    }
    let mut checkpoints = vec![Checkpoint {
        cycle: 0,
        process: memory::snapshot()?,
    }];
    for cycle in 1..=options.cycles {
        enable.push(switch(&options.mods, 8)?);
        restore.push(switch(&[], 3)?);
        if cycle % 16 == 0 || cycle == options.cycles {
            checkpoints.push(Checkpoint {
                cycle,
                process: memory::snapshot()?,
            });
        }
    }
    Ok(Measurements {
        enable,
        restore,
        checkpoints,
    })
}

pub fn run(args: impl Iterator<Item = OsString>) -> Result<()> {
    let options = Options::parse(args)?;
    let (_, game) = Manifest::read_game(&options.artifacts)?;
    let before_llvm = memory::snapshot()?;
    let dll = options
        .dll
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let api = Api::load(&dll)?;
    api.initialize()?;
    let after_llvm = memory::snapshot()?;
    let plan = HotReload::plan(&api, &game, &options.mods).map_err(|error| error.to_string())?;
    let indexed_targets = plan.index.targets.len() + plan.index.typed_return_targets.len();
    drop(plan);
    let mut runtime = HotReload::new(&api, &game, &[]).map_err(|error| error.to_string())?;
    let measurements = if options.background {
        thread::scope(|scope| {
            let mut runtime = runtime.background(scope);
            measure(&options, |mods, expected| {
                replace_in_background(&mut runtime, mods, expected)
            })
        })?
    } else {
        let measurements = measure(&options, |mods, expected| {
            replace(&mut runtime, mods, expected)
        })?;
        drop(runtime);
        measurements
    };
    let background_timings = options.background.then(|| {
        let samples = || measurements.enable.iter().chain(&measurements.restore);
        BackgroundTimings {
            request_pause: Pause::from_samples(samples().map(|sample| sample.request_ms).collect()),
            time_until_activation: Pause::from_samples(
                samples().map(|sample| sample.elapsed_ms).collect(),
            ),
            calls_on_old_generation: samples().map(|sample| sample.old_calls).sum(),
        }
    });
    let report = Report {
        background: options.background,
        game_sha256: sha256(&game),
        bitcode_bytes: game.len(),
        indexed_targets,
        mods: options.mods,
        warmup_cycles: WARMUP_CYCLES,
        measured_cycles: options.cycles,
        enable: Pause::from_samples(
            measurements
                .enable
                .into_iter()
                .map(|sample| sample.pause_ms)
                .collect(),
        ),
        restore_base: Pause::from_samples(
            measurements
                .restore
                .into_iter()
                .map(|sample| sample.pause_ms)
                .collect(),
        ),
        background_timings,
        memory_before_llvm: before_llvm,
        memory_after_llvm: after_llvm,
        memory_after_restore: measurements.checkpoints,
        memory_after_jit_disposal: memory::snapshot()?,
    };
    write_json(&options.report, &report)?;
    println!(
        "{} targets, {} cycles: enable p50/p95 {:.2}/{:.2} ms, restore p50/p95 {:.2}/{:.2} ms",
        report.indexed_targets,
        report.measured_cycles,
        report.enable.median_ms,
        report.enable.p95_ms,
        report.restore_base.median_ms,
        report.restore_base.p95_ms,
    );
    Ok(())
}
