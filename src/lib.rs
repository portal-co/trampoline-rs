use std::fmt;

#[derive(Debug)]
pub enum Rec<T> {
    Rtrn(T),
    Call(Thunk<Rec<T>>),
}

pub fn tail_call<T>(fun: impl FnOnce() -> Rec<T> + 'static) -> Rec<T> {
    Rec::Call(Thunk::new(fun))
}

pub fn ret<T>(val: T) -> Rec<T> {
    Rec::Rtrn(val)
}

pub fn rec<T>(start: impl FnOnce() -> Rec<T> + 'static) -> T {
    let mut res = start();
    loop {
        match res {
            Rec::Rtrn(x) => break x,
            Rec::Call(thunk) => res = thunk.compute(),
        }
    }
}

pub struct Thunk<T> {
    fun: Box<FnThunk<Out = T>>,
}

trait FnThunk {
    type Out;

    fn call_boxed(self: Box<Self>) -> Self::Out;
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
    pub fn new(fun: impl FnOnce() -> T + 'static) -> Self {
        Self {
            fun: Box::new(fun),
        }
    }

    pub fn compute(self) -> T {
        self.fun.call_boxed()
    }
}

impl<T> fmt::Debug for Thunk<T> {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "thunk {} ptr: {:?} {}", '{', &*self.fun as *const _, '}')
    }
}
