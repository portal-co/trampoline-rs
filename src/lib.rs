use std::fmt;

/// A single recursive-function result.
#[derive(Debug)]
pub enum Rec<T> {
    /// This variant is returned when the function is done; i.e. this the
    /// result of the computation.
    Ret(T),
    /// This variant is returned when the function is about to call itself
    /// or another in a tail position (i.e. there is no job after the call),
    /// generally indicates recursion.
    Call(Thunk<Rec<T>>),
}

trait FnThunk {
    type Out;

    fn call_boxed(self: Box<Self>) -> Self::Out;
}

/// A delayed computation. This can be used in lazy evaluation environments.
/// Also, it is used to delay a tail call and emulate TCO (tail call
/// optimization).
pub struct Thunk<T> {
    fun: Box<FnThunk<Out = T>>,
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

impl<T> Thunk<T> {
    /// Creates a new thunk from the given function. Probably you will end up
    /// passing closures to this function.
    pub fn new(fun: impl FnOnce() -> T + 'static) -> Self {
        Self {
            fun: Box::new(fun),
        }
    }

    /// Computes the result of this thunk, i.e. forces evaluation to happen.
    pub fn compute(self) -> T {
        self.fun.call_boxed()
    }
}

impl<T> fmt::Debug for Thunk<T> {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "thunk {} ptr: {:?} {}", '{', &*self.fun as *const _, '}')
    }
}

/// Given an input closure which returns the `Rec` type, this function performs
/// a trampolin over the returned value. While `Rec::Call(thunk)` is returned,
/// this function will keep evauating `thunk`. Whenever `Rec::Done(x)` is
/// found, `x` is returned.
pub fn trampolin<T>(start: impl FnOnce() -> Rec<T> + 'static) -> T {
    let mut res = start();
    loop {
        match res {
            Rec::Ret(x) => break x,
            Rec::Call(thunk) => res = thunk.compute(),
        }
    }
}

/// Turns a (probably recursive) tail call into a return value of
/// type `Rec`. The variant created is `Rec::Call`.
#[macro_export]
macro_rules! rec_call {
    ($call:expr) => {
        return $crate::Rec::Call($crate::Thunk::new(move || $call));
    };
}

/// Returns a value from a `Rec`-function. This means the recursion is done.
#[macro_export]
macro_rules! rec_ret {
    ($val:expr) => {
        return $crate::Rec::Ret($val);
    };
}

/// Executes a `Rec`-function call inside a trampolin.
#[macro_export]
macro_rules! trampolin {
    ($call:expr) => {
        $crate::trampolin(move || $call)
    };
}
