//! Futures.

use crate::task::{self, Poll};
use core::pin::PinMut;

pub use core::future::{Future, FutureObj, LocalFutureObj, UnsafeFutureObj};

/// A `Future` or `TryFuture` which tracks whether or not the underlying future
/// should no longer be polled.
///
/// `can_poll` will return `false` if a future should no longer be polled.
/// Usually, this state occurs after `poll` (or `try_poll`) returned
/// `Poll::Ready`. However, `can_poll` may also return `false` if a future
/// has become inactive and can no longer make progress and should be ignored
/// or dropped rather than being `poll`ed again.
pub trait FusedFuture {
    /// Returns `false` if the underlying future should no longer be polled.
    fn can_poll(&self) -> bool;
}

/// A convenience for futures that return `Result` values that includes
/// a variety of adapters tailored to such futures.
pub trait TryFuture {
    /// The type of successful values yielded by this future
    type Ok;

    /// The type of failures yielded by this future
    type Error;

    /// Poll this `TryFuture` as if it were a `Future`.
    ///
    /// This method is a stopgap for a compiler limitation that prevents us from
    /// directly inheriting from the `Future` trait; in the future it won't be
    /// needed.
    fn try_poll(
        self: PinMut<Self>,
        cx: &mut task::Context,
    ) -> Poll<Result<Self::Ok, Self::Error>>;
}

impl<F, T, E> TryFuture for F
    where F: Future<Output = Result<T, E>>
{
    type Ok = T;
    type Error = E;

    #[inline]
    fn try_poll(self: PinMut<Self>, cx: &mut task::Context) -> Poll<F::Output> {
        self.poll(cx)
    }
}
