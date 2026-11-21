//! Experimental author API. Bodies remain ordinary Rust compiled by rustc.
#![no_std]

use core::num::NonZeroU32;

pub use mixin_llvm_macros::{
    inject_after_call, inject_head, inject_return, modify_call_result, modify_head,
    modify_parameter, modify_return_result, targets,
};

/// A two-dimensional vector with an explicit layout for Mixin return values.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

const _: () = {
    assert!(core::mem::size_of::<Vec2>() == 8);
    assert!(core::mem::align_of::<Vec2>() == 4);
    assert!(core::mem::offset_of!(Vec2, x) == 0);
    assert!(core::mem::offset_of!(Vec2, y) == 4);
};

/// An entity identifier interpreted by the game. Zero represents a missing
/// identifier and is reserved for the ABI of `Option<EntityId>`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(NonZeroU32);

impl EntityId {
    #[inline(always)]
    pub const fn new(value: u32) -> Option<Self> {
        match NonZeroU32::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    #[inline(always)]
    pub const fn get(self) -> u32 {
        self.0.get()
    }
}
