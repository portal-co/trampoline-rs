//! This crate provides some facilities for recursing in Rust.
//!
//! # Motivation
//! Most iterative problems can be solved using recursion instead of loops.
//! In fact, when you start learning functional programming, you will see
//! the possibility of solving a given problem using recursion. If the language
//! does not optimize away some kinds of recursion, programming functionally
//! may be hard, because stack overflow may happen eventually if a big case of
//! recursion happens.
//!
//! A common optimization is the so called "tail call optimization", "tail
//! call reduction" or any similar name. First, let's understand tail call.
//! Tail call is basically a function found in "tail" position of a function
//! body. The tail position means the call is the last thing done in the
//! function body. Optimizing it away means that, instead of generating a
//! new stack frame for the called function, the compiler will reuse the
//! current frame for the called function. In fact, the call may be turned into
//! a loop. But it still is recursion, because the programmer did not write a
//! loop. It's just an optimization.
//!
//! Currently, Rust does not provide any language item to optimize the tail
//! call. There is a "promised" feature named `become`. `become`, for now,
//! is just a reserved identifer, but the intention is that it acts like a
//! return, but does a tail call reduction. Well, there is no prevision for
//! when this is implemented or if it ever will be implemented. So, we decided
//! to create a library allowing the programmer to write optimized recursion
//! with tail calls.
//!
//! # Example
//! Take a look in this factorial function:
//! ```rust
//! fn fac(n: u128) -> u128 {
//!     if n > 1 {
//!         n * fac(n - 1)
//!     } else {
//!         n
//!     }
//! }
//! ```
//! Clearly, the function above is not tail recursive. After the recursive
//! call, we still have to multiply the result by `n`. However, there is a
//! well-known version of this function which takes an extra argument: an
//! accumulator. Take a look:
//! ```rust
//! fn fac(n: u128) -> u128 {
//!     //!
//!     fn fac_with_acc(n: u128, acc: u128) -> u128 {
//!         if n > 1 {
//!             fac_with_acc(n - 1, acc * n)
//!         } else {
//!             1
//!         }
//!     }
//!
//!     fac_with_acc(n, 1)
//! }
//! ```
//! The function `fac_with_acc` is tail recursive. There is no job to be done
//! after the call to itself. There is only one problem: Rust does not do any
//! tail call optimization (yet). Now, the crate `tramp` does its job. We
//! implemented in this crate a trampoline. A trampoline simulates a tail
//! call optimization by calling a function which might return another function
//! (which will be called again) or a computed result. The trampoline will
//! keep calling in a loop the returned function, and whenever the function
//! returns a computed value instead of a function, the trampoline returns the
//! value. Take a look in the example below.
//!
//! ```rust
//! #[macro_use] extern crate tramp;
//!
//! use tramp::{tramp, Rec};
//!
//! fn factorial(n: u128) -> u128 {
//!     fn fac_with_acc(n: u128, acc: u128) -> Rec<u128> {
//!         if n > 1 {
//!             rec_call!(fac_with_acc(n - 1, acc * n))
//!         } else {
//!             rec_ret!(acc)
//!         }
//!     }
//!
//!     tramp(fac_with_acc(n, 1))
//! }
//!
//! assert_eq!(factorial(5), 120);
//! ```
#![no_std]
extern crate alloc;
use alloc::boxed::Box;
use core::fmt;

/// A single recursive-function result with static lifetime.
pub type Rec<T> = BorrowRec<'static, T>;

/// A single borrowed recursive-function result.
#[derive(Debug)]
pub enum BorrowRec<'a, T> {
    /// This variant is returned when the function is done; i.e. this the
    /// result of the computation.
    Ret(T),
    /// This variant is returned when the function is about to call itself
    /// or another in a tail position (i.e. there is no job after the call),
    /// generally indicates recursion.
    Call(Thunk<'a, BorrowRec<'a, T>>),
}

trait FnThunk {
    type Out;

    fn call_boxed(self: Box<Self>) -> Self::Out;
}

/// A delayed computation. This can be used in lazy evaluation environments.
/// Also, it is used to delay a tail call and emulate TCO (tail call
/// optimization).
pub struct Thunk<'a, T> {
    fun: Box<dyn FnThunk<Out = T> + 'a>,
}

impl<T, F> FnThunk for F
where
    F: FnOnce() -> T,
{
    type Out = T;

    fn call_boxed(self: Box<Self>) -> T {
        (*self)()
    }
}

impl<'a, T> Thunk<'a, T> {
    /// Creates a new thunk from the given function. Probably you will end up
    /// passing closures to this function.
    pub fn new(fun: impl FnOnce() -> T + 'a) -> Self {
        Self {
            fun: Box::new(fun),
        }
    }

    /// Computes the result of this thunk, i.e. forces evaluation to happen.
    pub fn compute(self) -> T {
        self.fun.call_boxed()
    }
}

impl<'a, T> fmt::Debug for Thunk<'a, T> {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "thunk {} ptr: {:?} {}", '{', &*self.fun as *const _, '}')
    }
}

/// Given an input which of type `BorrowRec`, this function performs
/// a trampoline over the value. While `Rec::Call(thunk)` is returned,
/// this function will keep evauating `thunk`. Whenever `Rec::Done(x)` is
/// found, `x` is returned.
pub fn tramp<'a, T>(mut res: BorrowRec<'a, T>) -> T {
    loop {
        match res {
            BorrowRec::Ret(x) => break x,
            BorrowRec::Call(thunk) => res = thunk.compute(),
        }
    }
}

/// Turns a (probably recursive) tail call into a return value of
/// type `Rec`. The variant created is `Rec::Call`.
/// Given an expression `x` of type `T`, then `rec_call!(x)` has type `Rec<T>`.
/// It is equivalent to, given a fictional attribute "optimize_tail_call",
/// `#[optimize_tail_call] return x` transforming `T` into `Rec<T>`.
#[macro_export]
macro_rules! rec_call {
    ($call:expr) => {
        return $crate::BorrowRec::Call($crate::Thunk::new(move || $call));
    };
}

/// Returns a value from a `Rec`-function. This means the recursion is done.
/// Given an expression `x` of type `T`, then `rec_ret!(x)` has type `Rec<T>`.
/// It is equivalent to `return x` transforming `T` into `Rec<T>`.
#[macro_export]
macro_rules! rec_ret {
    ($val:expr) => {
        return $crate::BorrowRec::Ret($val);
    };
}
