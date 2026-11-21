//! Ordinary Rust compiled both into the AOT runner and retained bitcode.
#![no_std]

/// Native ABI for the example host's synchronous damage event.
pub mod events {
    use core::ffi::c_void;

    pub type DamageCallback = extern "C" fn(f64) -> f64;
    pub type SubscribeDamage = unsafe extern "C" fn(*mut c_void, DamageCallback);

    /// The context and subscription function are valid only during registration.
    /// A registered callback must remain callable while its DLL is loaded.
    pub type RegisterPlugin = unsafe extern "C" fn(*mut c_void, SubscribeDamage);
    pub const REGISTER_PLUGIN_V1: &[u8] = b"mixin_register_v1\0";
}

#[mixin_sdk::targets("demo::combat")]
pub mod combat {
    pub fn armor_reduce(amount: f64, armor: f64) -> f64 {
        amount - armor * 0.5
    }

    pub fn apply_damage(amount: f64, armor: f64) -> f64 {
        armor_reduce(amount, armor)
    }

    pub fn two_hits(amount: f64, armor: f64) -> f64 {
        armor_reduce(amount, armor) + armor_reduce(amount + 2.0, armor)
    }

    pub fn caller(amount: f64, armor: f64) -> f64 {
        apply_damage(amount, armor) + 1.0
    }
}

pub extern "C" fn fixture_damage(amount: f64, armor: f64) -> f64 {
    combat::apply_damage(amount, armor)
}

pub extern "C" fn fixture_armor_reduce(amount: f64, armor: f64) -> f64 {
    combat::armor_reduce(amount, armor)
}

// This deliberately includes integer operations and a loop. rustc, rather
// than a custom expression parser, lowers all of it.
#[inline]
pub fn batch_with<F: FnMut(f64, f64) -> f64>(iterations: u64, seed: f64, mut f: F) -> f64 {
    let mut sum = 0.0;
    let mut i = 0;
    while i < iterations {
        let amount = seed + (i & 31) as f64 * 0.125;
        let armor = (i & 7) as f64;
        sum += f(amount, armor);
        i += 1;
    }
    sum
}

#[unsafe(no_mangle)]
pub extern "C" fn fixture_batch(iterations: u64, seed: f64) -> f64 {
    batch_with(iterations, seed, |amount, armor| {
        fixture_damage(amount, armor)
    })
}
