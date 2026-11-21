#![cfg_attr(mixin_retained, no_std)]

#[cfg(not(mixin_retained))]
use {core::ffi::c_void, llvm_fixture::events::SubscribeDamage};

#[cfg(not(mixin_retained))]
extern "C" fn on_damage(damage: f64) -> f64 {
    damage * 0.5
}

/// Register the damage callback with the host's event system.
///
/// # Safety
/// The host must supply the context and subscription function for this call.
#[cfg(not(mixin_retained))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mixin_register_v1(context: *mut c_void, subscribe: SubscribeDamage) {
    unsafe { subscribe(context, on_damage) };
}

#[cfg(mixin_retained)]
#[mixin_sdk::modify_parameter(
    id = "stronger_attack",
    target = "demo::combat::apply_damage",
    parameter = 0
)]
pub fn stronger_attack(amount: f64) -> f64 {
    amount + 4.0
}
