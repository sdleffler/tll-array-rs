use std::fmt;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::slice;

use tll::ternary::{Nat, Pred, NatPred, Triple, NatTriple, Zero, One, Two};
use tll_iterator::{SizedIterator, NonEmpty, FromSizedIterator};

use guillotine::*;
use storage::*;


/// The `array![]` macro provides a convenient way to construct `Array`s from scratch. It can be
/// invoked similarly to the `vec![]` macro, although `array![]` *does not* provide the "repeat"
/// syntax (like how `vec![0; 10]` would produce a `Vec` of 10 zeroes).
#[macro_export]
macro_rules! array {
    (@assign $data:ident $n:expr => $x:expr $(, $xs:expr)*) => (
        ::std::ptr::write(&mut $data[$n], $x); array!(@assign $data ($n + 1) => $($xs),*));
    (@assign $data:ident $n:expr =>) => ();
    (@count $x:expr $(, $xs:expr)*) => ($crate::tll::ternary::Succ<array!(@count $($xs),*)>);
    (@count) => ($crate::tll::ternary::Term);
    ($($xs:expr),*) => ({
        unsafe {
            let mut data =
                ::std::mem::uninitialized::<$crate::array::Array<array!(@count $($xs),*), _>>();
            array!(@assign data 0 => $($xs),*);
            data
        }
    });
}


/// The `Array` struct represents an array the length of which is determined by a type-level `Nat`.
/// For technical reasons, the `Arrayify<T>` trait is also necessary. Since `Arrayify<T>` includes
/// `Nat` as a supertrait, it will in most cases suffice to simply use the `Arrayify<T>` on an
/// `Array`'s length.
///
/// `Array`s dereference to slices to provide indexing operations. This means they can be treated
/// in much the same way one would treat a `Vec` (since `Vec`s work in much the same manner).
/// Eventually it may be necessary to extend these indexing operations to the `Array` struct
/// itself, but to say the least, that time is not now.
pub struct Array<L: Arrayify<T>, T> {
    data: Reify<L, T>,
}


impl<L: Arrayify<T>, T: Clone> Clone for Array<L, T>
    where Reify<L, T>: Clone
{
    fn clone(&self) -> Self {
        Array { data: self.data.clone() }
    }
}

impl<L: Arrayify<T>, T: Copy> Copy for Array<L, T> where Reify<L, T>: Copy {}


impl<L: Arrayify<T>, T> Deref for Array<L, T> {
    type Target = [T];

    fn deref<'a>(&'a self) -> &'a [T] {
        unsafe { slice::from_raw_parts(self as *const Self as *const T, L::reify()) }
    }
}

impl<L: Arrayify<T>, T> DerefMut for Array<L, T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut [T] {
        unsafe { slice::from_raw_parts_mut(self as *mut Self as *mut T, L::reify()) }
    }
}


pub trait ArraySplit<L: Arrayify<T> + NatPred, T>
    where Pred<L>: Arrayify<T>
{
    /// Split the array into its first element and remaining elements.
    fn split_first(Array<L, T>) -> (T, Array<Pred<L>, T>);

    /// Split the array into its first elements and last element.
    fn split_last(Array<L, T>) -> (T, Array<Pred<L>, T>);
}

impl<L: Arrayify<T> + NatPred, T> ArraySplit<Zero<L>, T> for Array<Zero<L>, T>
    where Pred<L>: Arrayify<T>
{
    fn split_first(array: Self) -> (T, Array<Pred<Zero<L>>, T>) {
        unsafe {
            let head = ptr::read((&array as *const Self as *const T));
            let tail = ptr::read((&array as *const Self as *const Array<Two<Pred<L>>, T>)
                .offset(1));
            mem::forget(array);
            (head, tail)
        }
    }

    fn split_last(array: Self) -> (T, Array<Pred<Zero<L>>, T>) {
        unsafe {
            let init = ptr::read(&array as *const Self as *const Array<Two<Pred<L>>, T>);
            let last = ptr::read((&array as *const Self as *const T)
                .offset(<Zero<L>>::reify() as isize - 1));
            mem::forget(array);
            (last, init)
        }
    }
}

impl<L: Arrayify<T> + NatPred + NatTriple, T> ArraySplit<One<L>, T> for Array<One<L>, T>
    where Pred<One<L>>: Arrayify<T>
{
    fn split_first(array: Self) -> (T, Array<Triple<L>, T>) {
        unsafe {
            let head = ptr::read((&array as *const Self as *const T));
            let tail = ptr::read((&array as *const Self as *const Array<Triple<L>, T>).offset(1));
            mem::forget(array);
            (head, tail)
        }
    }

    fn split_last(array: Self) -> (T, Array<Triple<L>, T>) {
        unsafe {
            let init = ptr::read(&array as *const Self as *const Array<Triple<L>, T>);
            let last = ptr::read((&array as *const Self as *const T)
                .offset(<One<L>>::reify() as isize - 1));
            mem::forget(array);
            (last, init)
        }
    }
}

impl<L: Arrayify<T> + NatPred, T> ArraySplit<Two<L>, T> for Array<Two<L>, T>
    where One<L>: Arrayify<T>
{
    fn split_first(array: Self) -> (T, Array<One<L>, T>) {
        unsafe {
            let head = ptr::read((&array as *const Self as *const T));
            let tail = ptr::read((&array as *const Self as *const Array<One<L>, T>).offset(1));
            mem::forget(array);
            (head, tail)
        }
    }

    fn split_last(array: Self) -> (T, Array<One<L>, T>) {
        unsafe {
            let init = ptr::read(&array as *const Self as *const Array<One<L>, T>);
            let last = ptr::read((&array as *const Self as *const T)
                .offset(<Two<L>>::reify() as isize - 1));
            mem::forget(array);
            (last, init)
        }
    }
}


impl<L: Arrayify<T>, T> Array<L, T> {
    /// Split the `Array` apart into its first element and an `Array` consisting of the remaining elements.
    /// This splitting is quite efficient and consists internally of only pointer casts and dereferences.
    ///
    /// ```
    /// # #[macro_use] extern crate tll_array; fn main() {
    /// let array = array![42i32, 84, 126, 168, 210, 252, 294, 336];
    /// assert_eq!(array.len(), 8);
    /// let (head, tail) = array.split_first();
    /// assert_eq!(head, 42);
    /// assert_eq!(tail.len(), 7);
    /// # }
    /// ```
    pub fn split_first(self) -> (T, Array<Pred<L>, T>)
        where Self: ArraySplit<L, T>,
              L: NatPred,
              Pred<L>: Arrayify<T>
    {
        <Self as ArraySplit<L, T>>::split_first(self)
    }

    /// Split the `Array` apart into its last element and an `Array` consisting of preceding elements.
    /// This splitting is quite efficient and consists internally of only pointer casts and dereferences.
    ///
    /// ```
    /// # #[macro_use] extern crate tll_array; fn main() {
    /// let array = array![42i32, 84, 126, 168, 210, 252, 294, 336];
    /// assert_eq!(array.len(), 8);
    /// let (last, init) = array.split_last();
    /// assert_eq!(last, 336);
    /// assert_eq!(init.len(), 7);
    /// # }
    /// ```
    pub fn split_last(self) -> (T, Array<Pred<L>, T>)
        where Self: ArraySplit<L, T>,
              L: NatPred,
              Pred<L>: Arrayify<T>
    {
        <Self as ArraySplit<L, T>>::split_last(self)
    }
}


pub struct ArrayIter<L: Arrayify<T>, T> {
    data: Guillotine<Array<L, T>>,
    pos: usize,
}

impl<L: Arrayify<T>, T> Drop for ArrayIter<L, T> {
    fn drop(&mut self) {
        unsafe {
            let mut data = self.data.take().unwrap_unchecked();
            for i in self.pos..data.len() {
                ptr::drop_in_place(&mut data[i]);
            }
        }
    }
}

impl<L: Arrayify<T>, T> Iterator for ArrayIter<L, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        unsafe {
            let data = self.data.as_ref().unwrap_unchecked();
            if self.pos < data.len() {
                let next = ptr::read(&data[self.pos]);
                self.pos += 1;
                Some(next)
            } else {
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (L::reify(), Some(L::reify()))
    }
}

impl<L: Arrayify<T>, T> IntoIterator for Array<L, T> {
    type IntoIter = ArrayIter<L, T>;
    type Item = T;

    fn into_iter(self) -> ArrayIter<L, T> {
        ArrayIter {
            data: Alive(self),
            pos: 0,
        }
    }
}

impl<L: Arrayify<T>, T> ExactSizeIterator for ArrayIter<L, T> {}


impl<L: Arrayify<T>, T: fmt::Debug> fmt::Debug for Array<L, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.deref().fmt(fmt)
    }
}


impl<L: Arrayify<T>, T> SizedIterator<L> for Array<L, T> {}

impl<L: Arrayify<T> + NatPred, T> NonEmpty<Zero<L>> for Array<Zero<L>, T>
    where Array<Zero<L>, T>: ArraySplit<Zero<L>, T>,
          Pred<L>: Arrayify<T>
{
    type Next = Array<Two<Pred<L>>, T>;

    fn next(self) -> (T, Array<Two<Pred<L>>, T>) {
        self.split_first()
    }
}

impl<L: Arrayify<T> + NatTriple, T> NonEmpty<One<L>> for Array<One<L>, T>
    where Array<One<L>, T>: ArraySplit<One<L>, T>,
          Triple<L>: Arrayify<T>
{
    type Next = Array<Triple<L>, T>;

    fn next(self) -> (T, Array<Triple<L>, T>) {
        self.split_first()
    }
}

impl<L: Arrayify<T>, T> NonEmpty<Two<L>> for Array<Two<L>, T>
    where Array<Two<L>, T>: ArraySplit<Two<L>, T>
{
    type Next = Array<One<L>, T>;

    fn next(self) -> (T, Array<One<L>, T>) {
        self.split_first()
    }
}

impl<L: Arrayify<T>, T> FromSizedIterator<L, T> for Array<L, T> {
    fn from_sized_iter<I: SizedIterator<L, Item = T>>(iter: I) -> Self {
        let mut array: Array<L, T>;
        unsafe {
            array = mem::uninitialized();
            for (i, v) in iter.into_iter().enumerate() {
                ptr::write(&mut array[i], v);
            }
        }
        array
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_first_1() {
        let array = array![42i32];
        assert_eq!(array.len(), 1);
        let (head, array) = array.split_first();
        assert_eq!(array.len(), 0);
        assert_eq!(head, 42);
    }

    #[test]
    fn split_first_2() {
        let array = array![42i32, 84];
        assert_eq!(array.len(), 2);
        let (head, array) = array.split_first();
        assert_eq!(array.len(), 1);
        assert_eq!(head, 42);
    }

    #[test]
    fn split_first_3() {
        let array = array![42i32, 84, 126];
        assert_eq!(array.len(), 3);
        let (head, array) = array.split_first();
        assert_eq!(array.len(), 2);
        assert_eq!(head, 42);
    }

    #[test]
    fn split_first_8() {
        let array = array![42i32, 84, 126, 168, 210, 252, 294, 336];
        assert_eq!(array.len(), 8);
        let (head, array) = array.split_first();
        assert_eq!(array.len(), 7);
        assert_eq!(head, 42);
    }

    #[test]
    fn split_first_9() {
        let array = array![42i32, 84, 126, 168, 210, 252, 294, 336, 378];
        assert_eq!(array.len(), 9);
        let (head, array) = array.split_first();
        assert_eq!(array.len(), 8);
        assert_eq!(head, 42);
    }

    #[test]
    fn split_first_10() {
        let array = array![42i32, 84, 126, 168, 210, 252, 294, 336, 378, 420];
        assert_eq!(array.len(), 10);
        let (head, array) = array.split_first();
        assert_eq!(array.len(), 9);
        assert_eq!(head, 42);
    }

    #[test]
    fn split_last_1() {
        let array = array![42i32];
        assert_eq!(array.len(), 1);
        let (tail, array) = array.split_last();
        assert_eq!(array.len(), 0);
        assert_eq!(tail, 42);
    }

    #[test]
    fn split_last_2() {
        let array = array![42i32, 84];
        assert_eq!(array.len(), 2);
        let (tail, array) = array.split_last();
        assert_eq!(array.len(), 1);
        assert_eq!(tail, 84);
    }

    #[test]
    fn split_last_3() {
        let array = array![42i32, 84, 126];
        assert_eq!(array.len(), 3);
        let (tail, array) = array.split_last();
        assert_eq!(array.len(), 2);
        assert_eq!(tail, 126);
    }

    #[test]
    fn split_last_8() {
        let array = array![42i32, 84, 126, 168, 210, 252, 294, 336];
        assert_eq!(array.len(), 8);
        let (tail, array) = array.split_last();
        assert_eq!(array.len(), 7);
        assert_eq!(tail, 336);
    }

    #[test]
    fn split_last_9() {
        let array = array![42i32, 84, 126, 168, 210, 252, 294, 336, 378];
        assert_eq!(array.len(), 9);
        let (tail, array) = array.split_last();
        assert_eq!(array.len(), 8);
        assert_eq!(tail, 378);
    }

    #[test]
    fn split_last_10() {
        let array = array![42i32, 84, 126, 168, 210, 252, 294, 336, 378, 420];
        assert_eq!(array.len(), 10);
        let (tail, array) = array.split_last();
        assert_eq!(array.len(), 9);
        assert_eq!(tail, 420);
    }
}
