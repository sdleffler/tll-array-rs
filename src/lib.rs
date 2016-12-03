//! # `tll-array`: Arrays with polymorphic, type-encoded length for Rust.
//!
//! The `tll-array` crate provides a workaround for Rust's current lack of support for constant
//! expressions. It provides an `Array<L: Nat, T>` type, where `L` is a type-level natural number
//! from the [`type-level-logic`](https://crates.io/crates/type-level-logic) crate. This `Array`
//! is `Copy` when `T` is `Copy`, `Clone` when `T` is `Clone`, and `Deref`s to `[T]`, providing
//! all familiar slice operations, safely bounds-checked.
//!
//! `Array` also implements some iterator machinery, as well as traits from
//! [`tll-iterator`](https://github.com/sdleffler/tll-iterator-rs), which provides iterators with
//! type-encoded lengths (similar to how this crate provides arrays with type-encoded lengths).
//! `SizedIterator<L, Item = T>`s can be `.collect_sized()` into `Array<L, T>`.

extern crate unreachable;

#[macro_use]
extern crate type_operators;
extern crate tll_iterator;

/// This is `pub` for the benefit of exported macros. This way, they can refer to it as `$crate::tll`.
pub extern crate type_level_logic as tll;

#[macro_use]
pub mod array;
mod guillotine;
mod storage;

pub use array::*;
