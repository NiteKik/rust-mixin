use mixin_runtime::{Jit, Result, bundle::Manifest};
use std::path::Path;
pub(crate) type DamageFn = extern "C" fn(f64, f64) -> f64;
pub(crate) type BatchFn = extern "C" fn(u64, f64) -> f64;
pub(crate) type HandlerFn = extern "C" fn(f64) -> f64;

pub(crate) fn read_fixture(root: &Path) -> Result<(Manifest, Vec<u8>, Vec<u8>)> {
    let (manifest, game) = Manifest::read_game(root)?;
    let handler = manifest.payload(root, "retained_mod.bc")?;
    Ok((manifest, game, handler))
}

pub(crate) extern "C" fn aot_patched_damage(amount: f64, armor: f64) -> f64 {
    llvm_handler::mod_adjust_damage(llvm_fixture::fixture_armor_reduce(amount, armor))
}

pub(crate) extern "C" fn aot_patched_batch(iterations: u64, seed: f64) -> f64 {
    llvm_fixture::batch_with(iterations, seed, |amount, armor| {
        aot_patched_damage(amount, armor)
    })
}

pub(crate) fn same_float(actual: f64, expected: f64, label: &str) -> Result<()> {
    if actual.to_bits() != expected.to_bits() && !(actual.is_nan() && expected.is_nan()) {
        return Err(format!(
            "{label}: got {actual:?} ({:x}), expected {expected:?} ({:x})",
            actual.to_bits(),
            expected.to_bits()
        ));
    }
    Ok(())
}

// Example-specific typed bindings live in the demo, not in the Mixin engine.
// All pointers are retained alongside the JIT owner and never leave this module.
pub(crate) struct Demo<'a> {
    pub(crate) _owner: Jit<'a>,
    pub(crate) damage: DamageFn,
    pub(crate) armor: DamageFn,
    pub(crate) batch: BatchFn,
    pub(crate) handler: HandlerFn,
}

impl<'a> Demo<'a> {
    pub(crate) fn new(jit: Jit<'a>) -> Result<Self> {
        let damage = &jit.index().target("demo::combat::apply_damage")?.entry;
        let armor = &jit.index().target("demo::combat::armor_reduce")?.entry;
        let handler = &jit
            .index()
            .handlers
            .first()
            .ok_or("missing demo handler")?
            .symbol;
        // SAFETY: macro-generated scalar bridges are validated by the loader;
        // fixture_batch's known signature comes from the trusted fixture source.
        // This struct keeps the code owner alive for every pointer invocation.
        unsafe {
            Ok(Self {
                damage: std::mem::transmute::<usize, DamageFn>(jit.lookup_raw(damage)?),
                armor: std::mem::transmute::<usize, DamageFn>(jit.lookup_raw(armor)?),
                batch: std::mem::transmute::<usize, BatchFn>(jit.lookup_raw("fixture_batch")?),
                handler: std::mem::transmute::<usize, HandlerFn>(jit.lookup_raw(handler)?),
                _owner: jit,
            })
        }
    }
}

pub(crate) fn verify(base: &Demo<'_>, patched: &Demo<'_>) -> Result<usize> {
    let mut checks = 0;
    let mut values = vec![
        0.0,
        -0.0,
        1.0,
        -1.0,
        10.0,
        20.0,
        f64::MIN_POSITIVE,
        f64::from_bits(1),
        f64::MAX,
        -f64::MAX,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
    ];
    // Deterministic finite inputs on both sides of the handler branch.
    for i in 0..4096_u64 {
        values.push((i as f64 - 2048.0) * 0.03125);
    }
    for (i, amount) in values.iter().copied().enumerate() {
        let armor = values[(i * 13 + 7) % values.len()];
        same_float(
            (base.damage)(amount, armor),
            llvm_fixture::fixture_damage(amount, armor),
            "base",
        )?;
        same_float(
            (patched.damage)(amount, armor),
            aot_patched_damage(amount, armor),
            "patched",
        )?;
        same_float(
            (patched.armor)(amount, armor),
            llvm_fixture::fixture_armor_reduce(amount, armor),
            "unmodified callee",
        )?;
        same_float(
            (base.handler)(amount),
            llvm_handler::mod_adjust_damage(amount),
            "Rust handler",
        )?;
        checks += 4;
    }
    for n in [0, 1, 2, 31, 32, 33, 127, 4096] {
        for seed in [-30.0, -0.0, 8.0, 20.0] {
            same_float(
                (base.batch)(n, seed),
                llvm_fixture::fixture_batch(n, seed),
                "base batch",
            )?;
            same_float(
                (patched.batch)(n, seed),
                aot_patched_batch(n, seed),
                "patched batch",
            )?;
            checks += 2;
        }
    }
    same_float((base.damage)(20.0, 8.0), 16.0, "base demo")?;
    same_float((patched.damage)(20.0, 8.0), 9.5, "patched demo")?;
    checks += 2;
    Ok(checks)
}
