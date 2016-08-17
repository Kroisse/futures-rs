#![allow(non_snake_case)]

use std::mem;

use {Future, Poll, IntoFuture};
use util::Collapsed;

macro_rules! generate {
    ($(($Join:ident, $new:ident, <A, $($B:ident),*>),)*) => ($(
        /// Future for the `join` combinator, waiting for two futures to
        /// complete.
        ///
        /// This is created by this `Future::join` method.
        pub struct $Join<A, $($B),*>
            where A: Future,
                  $($B: Future<Error=A::Error>),*
        {
            a: MaybeDone<A>,
            $($B: MaybeDone<$B>,)*
        }

        pub fn $new<A, $($B),*>(a: A, $($B: $B),*) -> $Join<A, $($B),*>
            where A: Future,
                  $($B: Future<Error=A::Error>),*
        {
            let a = Collapsed::Start(a);
            $(let $B = Collapsed::Start($B);)*
            $Join {
                a: MaybeDone::NotYet(a),
                $($B: MaybeDone::NotYet($B)),*
            }
        }

        impl<A, $($B),*> $Join<A, $($B),*>
            where A: Future,
                  $($B: Future<Error=A::Error>),*
        {
            fn erase(&mut self) {
                self.a = MaybeDone::Gone;
                $(self.$B = MaybeDone::Gone;)*
            }
        }

        impl<A, $($B),*> Future for $Join<A, $($B),*>
            where A: Future,
                  $($B: Future<Error=A::Error>),*
        {
            type Item = (A::Item, $($B::Item),*);
            type Error = A::Error;

            fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
                let mut all_done = match self.a.poll() {
                    Ok(done) => done,
                    Err(e) => {
                        self.erase();
                        return Poll::Err(e)
                    }
                };
                $(
                    all_done = match self.$B.poll() {
                        Ok(done) => all_done && done,
                        Err(e) => {
                            self.erase();
                            return Poll::Err(e)
                        }
                    };
                )*

                if all_done {
                    Poll::Ok((self.a.take(), $(self.$B.take()),*))
                } else {
                    Poll::NotReady
                }
            }

            unsafe fn tailcall(&mut self)
                               -> Option<Box<Future<Item=Self::Item, Error=Self::Error>>> {
                self.a.collapse();
                $(self.$B.collapse();)*
                None
            }
        }

        impl<A, $($B),*> IntoFuture for (A, $($B),*)
            where A: IntoFuture,
        $(
            $B: IntoFuture<Error=A::Error>
        ),*
        {
            type Future = $Join<A::Future, $($B::Future),*>;
            type Item = (A::Item, $($B::Item),*);
            type Error = A::Error;

            fn into_future(self) -> Self::Future {
                match self {
                    (a, $($B),+) => {
                        $new(
                            IntoFuture::into_future(a),
                            $(IntoFuture::into_future($B)),+
                        )
                    }
                }
            }
        }

    )*)
}

generate! {
    (Join, new, <A, B>),
    (Join3, new3, <A, B, C>),
    (Join4, new4, <A, B, C, D>),
    (Join5, new5, <A, B, C, D, E>),
}

enum MaybeDone<A: Future> {
    NotYet(Collapsed<A>),
    Done(A::Item),
    Gone,
}

impl<A: Future> MaybeDone<A> {
    fn poll(&mut self) -> Result<bool, A::Error> {
        let res = match *self {
            MaybeDone::NotYet(ref mut a) => a.poll(),
            MaybeDone::Done(_) => return Ok(true),
            MaybeDone::Gone => panic!("cannot poll Join twice"),
        };
        match res {
            Poll::Ok(res) => {
                *self = MaybeDone::Done(res);
                Ok(true)
            }
            Poll::Err(res) => Err(res),
            Poll::NotReady => Ok(false),
        }
    }

    fn take(&mut self) -> A::Item {
        match mem::replace(self, MaybeDone::Gone) {
            MaybeDone::Done(a) => a,
            _ => panic!(),
        }
    }

    unsafe fn collapse(&mut self) {
        if let MaybeDone::NotYet(ref mut a) = *self {
            a.collapse()
        }
    }
}
