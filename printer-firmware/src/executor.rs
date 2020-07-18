use core::future::Future;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use pin_utils::pin_mut;

const VTABLE: RawWakerVTable = {
    unsafe fn clone(s: *const ()) -> RawWaker {
        RawWaker::new(s, &VTABLE)
    }
    unsafe fn wake(_: *const ()) {}
    unsafe fn wake_by_ref(_: *const ()) {}
    unsafe fn drop(_: *const ()) {}

    RawWakerVTable::new(clone, wake, wake_by_ref, drop)
};

pub fn block_on<T>(t: T) -> T::Output
where
    T: Future,
{
    let raw_waker = RawWaker::new(core::ptr::null(), &VTABLE);
    pin_mut!(t);

    unsafe {
        let waker = Waker::from_raw(raw_waker);
        let mut ctx = Context::from_waker(&waker);

        loop {
            match t.as_mut().poll(&mut ctx) {
                Poll::Ready(out) => return out,
                Poll::Pending => {}
            }
        }
    }
}
