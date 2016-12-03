use std::mem;

use unreachable::unreachable;


/// The `Guillotine` type is a special variation on `Option` which is safe to use in the context
/// of manual drops and casting. It is used in the implementation of `Drop` for `ArrayIter`, since
/// sometimes not all elements of the array moved into the iterator will be used. In those cases,
/// it is necessary to drop *some but not all* of the elements in the iterator. As a result, a
/// `Guillotine` is used, and we `.take()` the value out of the struct field typed as
/// `Guillotine<Array<L, T>>`. This is necessary because Rust only gives us an `&mut self` to work
/// with when implementing `Drop`; as such, we have to `.take()` the value instead of moving the
/// whole struct into our `Drop` implementation.
#[repr(u8)]
pub enum Guillotine<T> {
    Alive(T),
    Dead,
}

pub use self::Guillotine::*;

impl<T> Guillotine<T> {
    pub fn take(&mut self) -> Guillotine<T> {
        mem::replace(self, Dead)
    }

    pub unsafe fn unwrap_unchecked(self) -> T {
        match self {
            Alive(obj) => obj,

            // The `unreachable` crate's `unreachable()` function provides
            // not just a hint to the compiler that this branch is
            // unreachable, but a static assertion! `.unwrap_unchecked()`
            // will cause undefined behavior if the `Guillotine` is in fact
            // `Dead`.
            Dead => unreachable(),
        }
    }

    pub fn as_ref(&self) -> Guillotine<&T> {
        match *self {
            Alive(ref obj) => Alive(obj),
            Dead => Dead,
        }
    }
}
