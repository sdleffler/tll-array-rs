//! The `storage` module contains types and type operators for building struct types which are
//! equivalent to contiguous blocks of memory for a given type. It works like this:
//!
//! - We start with an unsigned number in ternary representation. For example, 8, which is 22 in
//!   ternary.
//! - We then convert this to a tree of structs, all of which are marked `#[repr(C)]` to prevent
//!   Rust/the LLVM from optimizing their structure somehow. These trees are constructed from:
//!   - `TermNode`: this represents the terminal of our ternary linked-list. As such, it provides
//!     no storage.
//!   - `ZeroNode`: this represents the "three-times the left-hand side plus zero" digit from our
//!     ternary representation. As such, it takes its argument (which consists of the *next* set
//!     of digits) and stores three copies of it.
//!   - `OneNode` and `TwoNode`: like `ZeroNode`, these represent "three times the left hand side"
//!     of our ternary number, but also add one and two elements of the array's type, respectively.
//!
//! So, for our example of 8 = 22 base 3, we have a linked-list type-level ternary representation
//! of `Two<Two<Term>>`, which gets converted into `TwoNode<T, TwoNode<T, TermNode<T>>>`. This
//! results in two elements being allocated in the outer `TwoNode`, plus three instances of the
//! inner `TwoNode` - thus we get two elements in the outer, three times two elements in the inner,
//! giving us 8 elements as we expected. Sweet!
//!
//! These types are not intended for direct access. Instead, the `Array` type wraps them and
//! provides functionality for casting these structures into slices so they can be worked with as
//! the contiguous memory they were intended to be.

use tll::ternary::{Nat, Term, Zero, One, Two};

use std::marker::PhantomData;

#[cfg(feature = "specialization")]
use std::usize;


/// In the case that the "specialization" feature is passed, this type is used to stop compilation
/// if a bad array type is reified. The `KillItWithFire` type alias expands to an array which is
/// much too large for any architecture to handle, forcing Rust to halt compilation. If there's a
/// better way to do this, I really want to know. Don't hold out on me, eddyb.
#[cfg(feature = "specialization")]
#[cfg_attr(rustfmt, rustfmt_skip)]
pub type KillItWithFire = [u64; (usize::MAX,
                                 "Error: A non-Nat value has made its way into the generic \
                                  arrayifier!").0];


/// This is a marker trait denoting types which can be reified into an `Array`.
pub trait ToArray<T> {}


#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct TermNode<T>(PhantomData<T>);

#[derive(Default)]
#[repr(C)]
pub struct ZeroNode<T, N> {
    phantom: PhantomData<T>,
    next: [N; 3],
}

#[derive(Default)]
#[repr(C)]
pub struct OneNode<T, N> {
    first: T,
    next: [N; 3],
}

#[derive(Default)]
#[repr(C)]
pub struct TwoNode<T, N> {
    first: T,
    second: T,
    next: [N; 3],
}


impl<T> ToArray<T> for TermNode<T> {}
impl<T, N: ToArray<T>> ToArray<T> for ZeroNode<T, N> {}
impl<T, N: ToArray<T>> ToArray<T> for OneNode<T, N> {}
impl<T, N: ToArray<T>> ToArray<T> for TwoNode<T, N> {}


impl<T: Clone, N: ToArray<T> + Clone> Clone for ZeroNode<T, N> {
    fn clone(&self) -> Self {
        ZeroNode {
            phantom: PhantomData,
            next: [self.next[0].clone(), self.next[1].clone(), self.next[2].clone()],
        }
    }
}

impl<T: Clone, N: ToArray<T> + Clone> Clone for OneNode<T, N> {
    fn clone(&self) -> Self {
        OneNode {
            first: self.first.clone(),
            next: [self.next[0].clone(), self.next[1].clone(), self.next[2].clone()],
        }
    }
}

impl<T: Clone, N: ToArray<T> + Clone> Clone for TwoNode<T, N> {
    fn clone(&self) -> Self {
        TwoNode {
            first: self.first.clone(),
            second: self.second.clone(),
            next: [self.next[0].clone(), self.next[1].clone(), self.next[2].clone()],
        }
    }
}

impl<T: Copy, N: ToArray<T> + Copy> Copy for ZeroNode<T, N> {}
impl<T: Copy, N: ToArray<T> + Copy> Copy for OneNode<T, N> {}
impl<T: Copy, N: ToArray<T> + Copy> Copy for TwoNode<T, N> {}


type_operators! {
    [A, B, C, D, E]

    (Reify) Arrayify(Nat, T: Sized): (ToArray T) {
        forall (T: Sized) {
            [Term, T] => (TermNode T)
            forall (N: Nat) {
                [(Zero N), T] => (ZeroNode T (# N T))
                [(One N), T] => (OneNode T (# N T))
                [(Two N), T] => (TwoNode T (# N T))
                #[cfg(feature = "specialization")] {
                    {N, T} => KillItWithFire
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tll::ternary::*;

    use std::mem;

    #[test]
    fn array_memory_size_i8() {
        assert_eq!(mem::size_of::<Reify<U0, i8>>(), mem::size_of::<[i8; 0]>());
        assert_eq!(mem::size_of::<Reify<U1, i8>>(), mem::size_of::<[i8; 1]>());
        assert_eq!(mem::size_of::<Reify<U2, i8>>(), mem::size_of::<[i8; 2]>());
        assert_eq!(mem::size_of::<Reify<U3, i8>>(), mem::size_of::<[i8; 3]>());
        assert_eq!(mem::size_of::<Reify<U4, i8>>(), mem::size_of::<[i8; 4]>());
        assert_eq!(mem::size_of::<Reify<U5, i8>>(), mem::size_of::<[i8; 5]>());
        assert_eq!(mem::size_of::<Reify<U6, i8>>(), mem::size_of::<[i8; 6]>());
        assert_eq!(mem::size_of::<Reify<U7, i8>>(), mem::size_of::<[i8; 7]>());
        assert_eq!(mem::size_of::<Reify<U8, i8>>(), mem::size_of::<[i8; 8]>());
        assert_eq!(mem::size_of::<Reify<U9, i8>>(), mem::size_of::<[i8; 9]>());
        assert_eq!(mem::size_of::<Reify<U10, i8>>(), mem::size_of::<[i8; 10]>());
        assert_eq!(mem::size_of::<Reify<U11, i8>>(), mem::size_of::<[i8; 11]>());
        assert_eq!(mem::size_of::<Reify<U12, i8>>(), mem::size_of::<[i8; 12]>());
        assert_eq!(mem::size_of::<Reify<U13, i8>>(), mem::size_of::<[i8; 13]>());
        assert_eq!(mem::size_of::<Reify<U14, i8>>(), mem::size_of::<[i8; 14]>());
        assert_eq!(mem::size_of::<Reify<U15, i8>>(), mem::size_of::<[i8; 15]>());
        assert_eq!(mem::size_of::<Reify<U16, i8>>(), mem::size_of::<[i8; 16]>());
        assert_eq!(mem::size_of::<Reify<U17, i8>>(), mem::size_of::<[i8; 17]>());
        assert_eq!(mem::size_of::<Reify<U18, i8>>(), mem::size_of::<[i8; 18]>());
        assert_eq!(mem::size_of::<Reify<U19, i8>>(), mem::size_of::<[i8; 19]>());
        assert_eq!(mem::size_of::<Reify<U20, i8>>(), mem::size_of::<[i8; 20]>());
        assert_eq!(mem::size_of::<Reify<U21, i8>>(), mem::size_of::<[i8; 21]>());
        assert_eq!(mem::size_of::<Reify<U22, i8>>(), mem::size_of::<[i8; 22]>());
        assert_eq!(mem::size_of::<Reify<U23, i8>>(), mem::size_of::<[i8; 23]>());
        assert_eq!(mem::size_of::<Reify<U24, i8>>(), mem::size_of::<[i8; 24]>());
        assert_eq!(mem::size_of::<Reify<U25, i8>>(), mem::size_of::<[i8; 25]>());
        assert_eq!(mem::size_of::<Reify<U26, i8>>(), mem::size_of::<[i8; 26]>());
        assert_eq!(mem::size_of::<Reify<U27, i8>>(), mem::size_of::<[i8; 27]>());
        assert_eq!(mem::size_of::<Reify<U28, i8>>(), mem::size_of::<[i8; 28]>());
        assert_eq!(mem::size_of::<Reify<U29, i8>>(), mem::size_of::<[i8; 29]>());
        assert_eq!(mem::size_of::<Reify<U30, i8>>(), mem::size_of::<[i8; 30]>());
        assert_eq!(mem::size_of::<Reify<U31, i8>>(), mem::size_of::<[i8; 31]>());
        assert_eq!(mem::size_of::<Reify<U32, i8>>(), mem::size_of::<[i8; 32]>());
        assert_eq!(mem::size_of::<Reify<U33, i8>>(), mem::size_of::<[i8; 33]>());
        assert_eq!(mem::size_of::<Reify<U34, i8>>(), mem::size_of::<[i8; 34]>());
        assert_eq!(mem::size_of::<Reify<U35, i8>>(), mem::size_of::<[i8; 35]>());
        assert_eq!(mem::size_of::<Reify<U36, i8>>(), mem::size_of::<[i8; 36]>());
        assert_eq!(mem::size_of::<Reify<U37, i8>>(), mem::size_of::<[i8; 37]>());
        assert_eq!(mem::size_of::<Reify<U38, i8>>(), mem::size_of::<[i8; 38]>());
        assert_eq!(mem::size_of::<Reify<U39, i8>>(), mem::size_of::<[i8; 39]>());
        assert_eq!(mem::size_of::<Reify<U40, i8>>(), mem::size_of::<[i8; 40]>());
        assert_eq!(mem::size_of::<Reify<U41, i8>>(), mem::size_of::<[i8; 41]>());
        assert_eq!(mem::size_of::<Reify<U42, i8>>(), mem::size_of::<[i8; 42]>());
        assert_eq!(mem::size_of::<Reify<U43, i8>>(), mem::size_of::<[i8; 43]>());
        assert_eq!(mem::size_of::<Reify<U44, i8>>(), mem::size_of::<[i8; 44]>());
        assert_eq!(mem::size_of::<Reify<U45, i8>>(), mem::size_of::<[i8; 45]>());
        assert_eq!(mem::size_of::<Reify<U46, i8>>(), mem::size_of::<[i8; 46]>());
        assert_eq!(mem::size_of::<Reify<U47, i8>>(), mem::size_of::<[i8; 47]>());
        assert_eq!(mem::size_of::<Reify<U48, i8>>(), mem::size_of::<[i8; 48]>());
        assert_eq!(mem::size_of::<Reify<U49, i8>>(), mem::size_of::<[i8; 49]>());
        assert_eq!(mem::size_of::<Reify<U50, i8>>(), mem::size_of::<[i8; 50]>());
        assert_eq!(mem::size_of::<Reify<U51, i8>>(), mem::size_of::<[i8; 51]>());
        assert_eq!(mem::size_of::<Reify<U52, i8>>(), mem::size_of::<[i8; 52]>());
        assert_eq!(mem::size_of::<Reify<U53, i8>>(), mem::size_of::<[i8; 53]>());
        assert_eq!(mem::size_of::<Reify<U54, i8>>(), mem::size_of::<[i8; 54]>());
        assert_eq!(mem::size_of::<Reify<U55, i8>>(), mem::size_of::<[i8; 55]>());
        assert_eq!(mem::size_of::<Reify<U56, i8>>(), mem::size_of::<[i8; 56]>());
        assert_eq!(mem::size_of::<Reify<U57, i8>>(), mem::size_of::<[i8; 57]>());
        assert_eq!(mem::size_of::<Reify<U58, i8>>(), mem::size_of::<[i8; 58]>());
        assert_eq!(mem::size_of::<Reify<U59, i8>>(), mem::size_of::<[i8; 59]>());
        assert_eq!(mem::size_of::<Reify<U60, i8>>(), mem::size_of::<[i8; 60]>());
        assert_eq!(mem::size_of::<Reify<U61, i8>>(), mem::size_of::<[i8; 61]>());
        assert_eq!(mem::size_of::<Reify<U62, i8>>(), mem::size_of::<[i8; 62]>());
        assert_eq!(mem::size_of::<Reify<U63, i8>>(), mem::size_of::<[i8; 63]>());
    }

    #[test]
    fn array_memory_size_i16() {
        assert_eq!(mem::size_of::<Reify<U0, i16>>(), mem::size_of::<[i16; 0]>());
        assert_eq!(mem::size_of::<Reify<U1, i16>>(), mem::size_of::<[i16; 1]>());
        assert_eq!(mem::size_of::<Reify<U2, i16>>(), mem::size_of::<[i16; 2]>());
        assert_eq!(mem::size_of::<Reify<U3, i16>>(), mem::size_of::<[i16; 3]>());
        assert_eq!(mem::size_of::<Reify<U4, i16>>(), mem::size_of::<[i16; 4]>());
        assert_eq!(mem::size_of::<Reify<U5, i16>>(), mem::size_of::<[i16; 5]>());
        assert_eq!(mem::size_of::<Reify<U6, i16>>(), mem::size_of::<[i16; 6]>());
        assert_eq!(mem::size_of::<Reify<U7, i16>>(), mem::size_of::<[i16; 7]>());
        assert_eq!(mem::size_of::<Reify<U8, i16>>(), mem::size_of::<[i16; 8]>());
        assert_eq!(mem::size_of::<Reify<U9, i16>>(), mem::size_of::<[i16; 9]>());
        assert_eq!(mem::size_of::<Reify<U10, i16>>(),
                   mem::size_of::<[i16; 10]>());
        assert_eq!(mem::size_of::<Reify<U11, i16>>(),
                   mem::size_of::<[i16; 11]>());
        assert_eq!(mem::size_of::<Reify<U12, i16>>(),
                   mem::size_of::<[i16; 12]>());
        assert_eq!(mem::size_of::<Reify<U13, i16>>(),
                   mem::size_of::<[i16; 13]>());
        assert_eq!(mem::size_of::<Reify<U14, i16>>(),
                   mem::size_of::<[i16; 14]>());
        assert_eq!(mem::size_of::<Reify<U15, i16>>(),
                   mem::size_of::<[i16; 15]>());
        assert_eq!(mem::size_of::<Reify<U16, i16>>(),
                   mem::size_of::<[i16; 16]>());
        assert_eq!(mem::size_of::<Reify<U17, i16>>(),
                   mem::size_of::<[i16; 17]>());
        assert_eq!(mem::size_of::<Reify<U18, i16>>(),
                   mem::size_of::<[i16; 18]>());
        assert_eq!(mem::size_of::<Reify<U19, i16>>(),
                   mem::size_of::<[i16; 19]>());
        assert_eq!(mem::size_of::<Reify<U20, i16>>(),
                   mem::size_of::<[i16; 20]>());
        assert_eq!(mem::size_of::<Reify<U21, i16>>(),
                   mem::size_of::<[i16; 21]>());
        assert_eq!(mem::size_of::<Reify<U22, i16>>(),
                   mem::size_of::<[i16; 22]>());
        assert_eq!(mem::size_of::<Reify<U23, i16>>(),
                   mem::size_of::<[i16; 23]>());
        assert_eq!(mem::size_of::<Reify<U24, i16>>(),
                   mem::size_of::<[i16; 24]>());
        assert_eq!(mem::size_of::<Reify<U25, i16>>(),
                   mem::size_of::<[i16; 25]>());
        assert_eq!(mem::size_of::<Reify<U26, i16>>(),
                   mem::size_of::<[i16; 26]>());
        assert_eq!(mem::size_of::<Reify<U27, i16>>(),
                   mem::size_of::<[i16; 27]>());
        assert_eq!(mem::size_of::<Reify<U28, i16>>(),
                   mem::size_of::<[i16; 28]>());
        assert_eq!(mem::size_of::<Reify<U29, i16>>(),
                   mem::size_of::<[i16; 29]>());
        assert_eq!(mem::size_of::<Reify<U30, i16>>(),
                   mem::size_of::<[i16; 30]>());
        assert_eq!(mem::size_of::<Reify<U31, i16>>(),
                   mem::size_of::<[i16; 31]>());
        assert_eq!(mem::size_of::<Reify<U32, i16>>(),
                   mem::size_of::<[i16; 32]>());
        assert_eq!(mem::size_of::<Reify<U33, i16>>(),
                   mem::size_of::<[i16; 33]>());
        assert_eq!(mem::size_of::<Reify<U34, i16>>(),
                   mem::size_of::<[i16; 34]>());
        assert_eq!(mem::size_of::<Reify<U35, i16>>(),
                   mem::size_of::<[i16; 35]>());
        assert_eq!(mem::size_of::<Reify<U36, i16>>(),
                   mem::size_of::<[i16; 36]>());
        assert_eq!(mem::size_of::<Reify<U37, i16>>(),
                   mem::size_of::<[i16; 37]>());
        assert_eq!(mem::size_of::<Reify<U38, i16>>(),
                   mem::size_of::<[i16; 38]>());
        assert_eq!(mem::size_of::<Reify<U39, i16>>(),
                   mem::size_of::<[i16; 39]>());
        assert_eq!(mem::size_of::<Reify<U40, i16>>(),
                   mem::size_of::<[i16; 40]>());
        assert_eq!(mem::size_of::<Reify<U41, i16>>(),
                   mem::size_of::<[i16; 41]>());
        assert_eq!(mem::size_of::<Reify<U42, i16>>(),
                   mem::size_of::<[i16; 42]>());
        assert_eq!(mem::size_of::<Reify<U43, i16>>(),
                   mem::size_of::<[i16; 43]>());
        assert_eq!(mem::size_of::<Reify<U44, i16>>(),
                   mem::size_of::<[i16; 44]>());
        assert_eq!(mem::size_of::<Reify<U45, i16>>(),
                   mem::size_of::<[i16; 45]>());
        assert_eq!(mem::size_of::<Reify<U46, i16>>(),
                   mem::size_of::<[i16; 46]>());
        assert_eq!(mem::size_of::<Reify<U47, i16>>(),
                   mem::size_of::<[i16; 47]>());
        assert_eq!(mem::size_of::<Reify<U48, i16>>(),
                   mem::size_of::<[i16; 48]>());
        assert_eq!(mem::size_of::<Reify<U49, i16>>(),
                   mem::size_of::<[i16; 49]>());
        assert_eq!(mem::size_of::<Reify<U50, i16>>(),
                   mem::size_of::<[i16; 50]>());
        assert_eq!(mem::size_of::<Reify<U51, i16>>(),
                   mem::size_of::<[i16; 51]>());
        assert_eq!(mem::size_of::<Reify<U52, i16>>(),
                   mem::size_of::<[i16; 52]>());
        assert_eq!(mem::size_of::<Reify<U53, i16>>(),
                   mem::size_of::<[i16; 53]>());
        assert_eq!(mem::size_of::<Reify<U54, i16>>(),
                   mem::size_of::<[i16; 54]>());
        assert_eq!(mem::size_of::<Reify<U55, i16>>(),
                   mem::size_of::<[i16; 55]>());
        assert_eq!(mem::size_of::<Reify<U56, i16>>(),
                   mem::size_of::<[i16; 56]>());
        assert_eq!(mem::size_of::<Reify<U57, i16>>(),
                   mem::size_of::<[i16; 57]>());
        assert_eq!(mem::size_of::<Reify<U58, i16>>(),
                   mem::size_of::<[i16; 58]>());
        assert_eq!(mem::size_of::<Reify<U59, i16>>(),
                   mem::size_of::<[i16; 59]>());
        assert_eq!(mem::size_of::<Reify<U60, i16>>(),
                   mem::size_of::<[i16; 60]>());
        assert_eq!(mem::size_of::<Reify<U61, i16>>(),
                   mem::size_of::<[i16; 61]>());
        assert_eq!(mem::size_of::<Reify<U62, i16>>(),
                   mem::size_of::<[i16; 62]>());
        assert_eq!(mem::size_of::<Reify<U63, i16>>(),
                   mem::size_of::<[i16; 63]>());
    }

    #[test]
    fn array_memory_size_i32() {
        assert_eq!(mem::size_of::<Reify<U0, i32>>(), mem::size_of::<[i32; 0]>());
        assert_eq!(mem::size_of::<Reify<U1, i32>>(), mem::size_of::<[i32; 1]>());
        assert_eq!(mem::size_of::<Reify<U2, i32>>(), mem::size_of::<[i32; 2]>());
        assert_eq!(mem::size_of::<Reify<U3, i32>>(), mem::size_of::<[i32; 3]>());
        assert_eq!(mem::size_of::<Reify<U4, i32>>(), mem::size_of::<[i32; 4]>());
        assert_eq!(mem::size_of::<Reify<U5, i32>>(), mem::size_of::<[i32; 5]>());
        assert_eq!(mem::size_of::<Reify<U6, i32>>(), mem::size_of::<[i32; 6]>());
        assert_eq!(mem::size_of::<Reify<U7, i32>>(), mem::size_of::<[i32; 7]>());
        assert_eq!(mem::size_of::<Reify<U8, i32>>(), mem::size_of::<[i32; 8]>());
        assert_eq!(mem::size_of::<Reify<U9, i32>>(), mem::size_of::<[i32; 9]>());
        assert_eq!(mem::size_of::<Reify<U10, i32>>(),
                   mem::size_of::<[i32; 10]>());
        assert_eq!(mem::size_of::<Reify<U11, i32>>(),
                   mem::size_of::<[i32; 11]>());
        assert_eq!(mem::size_of::<Reify<U12, i32>>(),
                   mem::size_of::<[i32; 12]>());
        assert_eq!(mem::size_of::<Reify<U13, i32>>(),
                   mem::size_of::<[i32; 13]>());
        assert_eq!(mem::size_of::<Reify<U14, i32>>(),
                   mem::size_of::<[i32; 14]>());
        assert_eq!(mem::size_of::<Reify<U15, i32>>(),
                   mem::size_of::<[i32; 15]>());
        assert_eq!(mem::size_of::<Reify<U16, i32>>(),
                   mem::size_of::<[i32; 16]>());
        assert_eq!(mem::size_of::<Reify<U17, i32>>(),
                   mem::size_of::<[i32; 17]>());
        assert_eq!(mem::size_of::<Reify<U18, i32>>(),
                   mem::size_of::<[i32; 18]>());
        assert_eq!(mem::size_of::<Reify<U19, i32>>(),
                   mem::size_of::<[i32; 19]>());
        assert_eq!(mem::size_of::<Reify<U20, i32>>(),
                   mem::size_of::<[i32; 20]>());
        assert_eq!(mem::size_of::<Reify<U21, i32>>(),
                   mem::size_of::<[i32; 21]>());
        assert_eq!(mem::size_of::<Reify<U22, i32>>(),
                   mem::size_of::<[i32; 22]>());
        assert_eq!(mem::size_of::<Reify<U23, i32>>(),
                   mem::size_of::<[i32; 23]>());
        assert_eq!(mem::size_of::<Reify<U24, i32>>(),
                   mem::size_of::<[i32; 24]>());
        assert_eq!(mem::size_of::<Reify<U25, i32>>(),
                   mem::size_of::<[i32; 25]>());
        assert_eq!(mem::size_of::<Reify<U26, i32>>(),
                   mem::size_of::<[i32; 26]>());
        assert_eq!(mem::size_of::<Reify<U27, i32>>(),
                   mem::size_of::<[i32; 27]>());
        assert_eq!(mem::size_of::<Reify<U28, i32>>(),
                   mem::size_of::<[i32; 28]>());
        assert_eq!(mem::size_of::<Reify<U29, i32>>(),
                   mem::size_of::<[i32; 29]>());
        assert_eq!(mem::size_of::<Reify<U30, i32>>(),
                   mem::size_of::<[i32; 30]>());
        assert_eq!(mem::size_of::<Reify<U31, i32>>(),
                   mem::size_of::<[i32; 31]>());
        assert_eq!(mem::size_of::<Reify<U32, i32>>(),
                   mem::size_of::<[i32; 32]>());
        assert_eq!(mem::size_of::<Reify<U33, i32>>(),
                   mem::size_of::<[i32; 33]>());
        assert_eq!(mem::size_of::<Reify<U34, i32>>(),
                   mem::size_of::<[i32; 34]>());
        assert_eq!(mem::size_of::<Reify<U35, i32>>(),
                   mem::size_of::<[i32; 35]>());
        assert_eq!(mem::size_of::<Reify<U36, i32>>(),
                   mem::size_of::<[i32; 36]>());
        assert_eq!(mem::size_of::<Reify<U37, i32>>(),
                   mem::size_of::<[i32; 37]>());
        assert_eq!(mem::size_of::<Reify<U38, i32>>(),
                   mem::size_of::<[i32; 38]>());
        assert_eq!(mem::size_of::<Reify<U39, i32>>(),
                   mem::size_of::<[i32; 39]>());
        assert_eq!(mem::size_of::<Reify<U40, i32>>(),
                   mem::size_of::<[i32; 40]>());
        assert_eq!(mem::size_of::<Reify<U41, i32>>(),
                   mem::size_of::<[i32; 41]>());
        assert_eq!(mem::size_of::<Reify<U42, i32>>(),
                   mem::size_of::<[i32; 42]>());
        assert_eq!(mem::size_of::<Reify<U43, i32>>(),
                   mem::size_of::<[i32; 43]>());
        assert_eq!(mem::size_of::<Reify<U44, i32>>(),
                   mem::size_of::<[i32; 44]>());
        assert_eq!(mem::size_of::<Reify<U45, i32>>(),
                   mem::size_of::<[i32; 45]>());
        assert_eq!(mem::size_of::<Reify<U46, i32>>(),
                   mem::size_of::<[i32; 46]>());
        assert_eq!(mem::size_of::<Reify<U47, i32>>(),
                   mem::size_of::<[i32; 47]>());
        assert_eq!(mem::size_of::<Reify<U48, i32>>(),
                   mem::size_of::<[i32; 48]>());
        assert_eq!(mem::size_of::<Reify<U49, i32>>(),
                   mem::size_of::<[i32; 49]>());
        assert_eq!(mem::size_of::<Reify<U50, i32>>(),
                   mem::size_of::<[i32; 50]>());
        assert_eq!(mem::size_of::<Reify<U51, i32>>(),
                   mem::size_of::<[i32; 51]>());
        assert_eq!(mem::size_of::<Reify<U52, i32>>(),
                   mem::size_of::<[i32; 52]>());
        assert_eq!(mem::size_of::<Reify<U53, i32>>(),
                   mem::size_of::<[i32; 53]>());
        assert_eq!(mem::size_of::<Reify<U54, i32>>(),
                   mem::size_of::<[i32; 54]>());
        assert_eq!(mem::size_of::<Reify<U55, i32>>(),
                   mem::size_of::<[i32; 55]>());
        assert_eq!(mem::size_of::<Reify<U56, i32>>(),
                   mem::size_of::<[i32; 56]>());
        assert_eq!(mem::size_of::<Reify<U57, i32>>(),
                   mem::size_of::<[i32; 57]>());
        assert_eq!(mem::size_of::<Reify<U58, i32>>(),
                   mem::size_of::<[i32; 58]>());
        assert_eq!(mem::size_of::<Reify<U59, i32>>(),
                   mem::size_of::<[i32; 59]>());
        assert_eq!(mem::size_of::<Reify<U60, i32>>(),
                   mem::size_of::<[i32; 60]>());
        assert_eq!(mem::size_of::<Reify<U61, i32>>(),
                   mem::size_of::<[i32; 61]>());
        assert_eq!(mem::size_of::<Reify<U62, i32>>(),
                   mem::size_of::<[i32; 62]>());
        assert_eq!(mem::size_of::<Reify<U63, i32>>(),
                   mem::size_of::<[i32; 63]>());
    }

    #[test]
    fn array_memory_size_i64() {
        assert_eq!(mem::size_of::<Reify<U0, i64>>(), mem::size_of::<[i64; 0]>());
        assert_eq!(mem::size_of::<Reify<U1, i64>>(), mem::size_of::<[i64; 1]>());
        assert_eq!(mem::size_of::<Reify<U2, i64>>(), mem::size_of::<[i64; 2]>());
        assert_eq!(mem::size_of::<Reify<U3, i64>>(), mem::size_of::<[i64; 3]>());
        assert_eq!(mem::size_of::<Reify<U4, i64>>(), mem::size_of::<[i64; 4]>());
        assert_eq!(mem::size_of::<Reify<U5, i64>>(), mem::size_of::<[i64; 5]>());
        assert_eq!(mem::size_of::<Reify<U6, i64>>(), mem::size_of::<[i64; 6]>());
        assert_eq!(mem::size_of::<Reify<U7, i64>>(), mem::size_of::<[i64; 7]>());
        assert_eq!(mem::size_of::<Reify<U8, i64>>(), mem::size_of::<[i64; 8]>());
        assert_eq!(mem::size_of::<Reify<U9, i64>>(), mem::size_of::<[i64; 9]>());
        assert_eq!(mem::size_of::<Reify<U10, i64>>(),
                   mem::size_of::<[i64; 10]>());
        assert_eq!(mem::size_of::<Reify<U11, i64>>(),
                   mem::size_of::<[i64; 11]>());
        assert_eq!(mem::size_of::<Reify<U12, i64>>(),
                   mem::size_of::<[i64; 12]>());
        assert_eq!(mem::size_of::<Reify<U13, i64>>(),
                   mem::size_of::<[i64; 13]>());
        assert_eq!(mem::size_of::<Reify<U14, i64>>(),
                   mem::size_of::<[i64; 14]>());
        assert_eq!(mem::size_of::<Reify<U15, i64>>(),
                   mem::size_of::<[i64; 15]>());
        assert_eq!(mem::size_of::<Reify<U16, i64>>(),
                   mem::size_of::<[i64; 16]>());
        assert_eq!(mem::size_of::<Reify<U17, i64>>(),
                   mem::size_of::<[i64; 17]>());
        assert_eq!(mem::size_of::<Reify<U18, i64>>(),
                   mem::size_of::<[i64; 18]>());
        assert_eq!(mem::size_of::<Reify<U19, i64>>(),
                   mem::size_of::<[i64; 19]>());
        assert_eq!(mem::size_of::<Reify<U20, i64>>(),
                   mem::size_of::<[i64; 20]>());
        assert_eq!(mem::size_of::<Reify<U21, i64>>(),
                   mem::size_of::<[i64; 21]>());
        assert_eq!(mem::size_of::<Reify<U22, i64>>(),
                   mem::size_of::<[i64; 22]>());
        assert_eq!(mem::size_of::<Reify<U23, i64>>(),
                   mem::size_of::<[i64; 23]>());
        assert_eq!(mem::size_of::<Reify<U24, i64>>(),
                   mem::size_of::<[i64; 24]>());
        assert_eq!(mem::size_of::<Reify<U25, i64>>(),
                   mem::size_of::<[i64; 25]>());
        assert_eq!(mem::size_of::<Reify<U26, i64>>(),
                   mem::size_of::<[i64; 26]>());
        assert_eq!(mem::size_of::<Reify<U27, i64>>(),
                   mem::size_of::<[i64; 27]>());
        assert_eq!(mem::size_of::<Reify<U28, i64>>(),
                   mem::size_of::<[i64; 28]>());
        assert_eq!(mem::size_of::<Reify<U29, i64>>(),
                   mem::size_of::<[i64; 29]>());
        assert_eq!(mem::size_of::<Reify<U30, i64>>(),
                   mem::size_of::<[i64; 30]>());
        assert_eq!(mem::size_of::<Reify<U31, i64>>(),
                   mem::size_of::<[i64; 31]>());
        assert_eq!(mem::size_of::<Reify<U32, i64>>(),
                   mem::size_of::<[i64; 32]>());
        assert_eq!(mem::size_of::<Reify<U33, i64>>(),
                   mem::size_of::<[i64; 33]>());
        assert_eq!(mem::size_of::<Reify<U34, i64>>(),
                   mem::size_of::<[i64; 34]>());
        assert_eq!(mem::size_of::<Reify<U35, i64>>(),
                   mem::size_of::<[i64; 35]>());
        assert_eq!(mem::size_of::<Reify<U36, i64>>(),
                   mem::size_of::<[i64; 36]>());
        assert_eq!(mem::size_of::<Reify<U37, i64>>(),
                   mem::size_of::<[i64; 37]>());
        assert_eq!(mem::size_of::<Reify<U38, i64>>(),
                   mem::size_of::<[i64; 38]>());
        assert_eq!(mem::size_of::<Reify<U39, i64>>(),
                   mem::size_of::<[i64; 39]>());
        assert_eq!(mem::size_of::<Reify<U40, i64>>(),
                   mem::size_of::<[i64; 40]>());
        assert_eq!(mem::size_of::<Reify<U41, i64>>(),
                   mem::size_of::<[i64; 41]>());
        assert_eq!(mem::size_of::<Reify<U42, i64>>(),
                   mem::size_of::<[i64; 42]>());
        assert_eq!(mem::size_of::<Reify<U43, i64>>(),
                   mem::size_of::<[i64; 43]>());
        assert_eq!(mem::size_of::<Reify<U44, i64>>(),
                   mem::size_of::<[i64; 44]>());
        assert_eq!(mem::size_of::<Reify<U45, i64>>(),
                   mem::size_of::<[i64; 45]>());
        assert_eq!(mem::size_of::<Reify<U46, i64>>(),
                   mem::size_of::<[i64; 46]>());
        assert_eq!(mem::size_of::<Reify<U47, i64>>(),
                   mem::size_of::<[i64; 47]>());
        assert_eq!(mem::size_of::<Reify<U48, i64>>(),
                   mem::size_of::<[i64; 48]>());
        assert_eq!(mem::size_of::<Reify<U49, i64>>(),
                   mem::size_of::<[i64; 49]>());
        assert_eq!(mem::size_of::<Reify<U50, i64>>(),
                   mem::size_of::<[i64; 50]>());
        assert_eq!(mem::size_of::<Reify<U51, i64>>(),
                   mem::size_of::<[i64; 51]>());
        assert_eq!(mem::size_of::<Reify<U52, i64>>(),
                   mem::size_of::<[i64; 52]>());
        assert_eq!(mem::size_of::<Reify<U53, i64>>(),
                   mem::size_of::<[i64; 53]>());
        assert_eq!(mem::size_of::<Reify<U54, i64>>(),
                   mem::size_of::<[i64; 54]>());
        assert_eq!(mem::size_of::<Reify<U55, i64>>(),
                   mem::size_of::<[i64; 55]>());
        assert_eq!(mem::size_of::<Reify<U56, i64>>(),
                   mem::size_of::<[i64; 56]>());
        assert_eq!(mem::size_of::<Reify<U57, i64>>(),
                   mem::size_of::<[i64; 57]>());
        assert_eq!(mem::size_of::<Reify<U58, i64>>(),
                   mem::size_of::<[i64; 58]>());
        assert_eq!(mem::size_of::<Reify<U59, i64>>(),
                   mem::size_of::<[i64; 59]>());
        assert_eq!(mem::size_of::<Reify<U60, i64>>(),
                   mem::size_of::<[i64; 60]>());
        assert_eq!(mem::size_of::<Reify<U61, i64>>(),
                   mem::size_of::<[i64; 61]>());
        assert_eq!(mem::size_of::<Reify<U62, i64>>(),
                   mem::size_of::<[i64; 62]>());
        assert_eq!(mem::size_of::<Reify<U63, i64>>(),
                   mem::size_of::<[i64; 63]>());
    }
}
