#![cfg_attr(feature = "nightly", feature(try_trait_v2))]

pub mod lib {
    pub use futures;

    pub mod lift {
        trait Lift<Lifted: super::try_trait::Try> {
            fn lift(self) -> Lifted;
        }
        impl<T: super::try_trait::Try> Lift<T> for T {
            fn lift(self) -> T {
                self
            }
        }

        impl<T> Lift<Option<T>> for T {
            fn lift(self) -> Option<T> {
                Some(self)
            }
        }

        impl<T, E> Lift<Result<T, E>> for T {
            fn lift(self) -> Result<T, E> {
                Ok(self)
            }
        }
    }

    #[cfg(feature = "nightly")]
    pub mod try_trait {
        pub use std::ops::Try;
    }

    #[cfg(not(feature = "nightly"))]
    pub mod try_trait {
        use std::ops::ControlFlow;
        use std::task::Poll;

        pub trait Try {
            type Output;
            type Residual;

            fn from_output(output: Self::Output) -> Self;

            fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
        }

        impl<B, C> Try for ControlFlow<B, C> {
            type Output = C;
            type Residual = B;

            fn from_output(output: Self::Output) -> Self {
                ControlFlow::Continue(output)
            }

            fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
                self
            }
        }

        impl<T> Try for Option<T> {
            type Output = T;
            type Residual = ();

            fn from_output(output: Self::Output) -> Self {
                Some(output)
            }

            fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
                match self {
                    Some(x) => ControlFlow::Continue(x),
                    None => ControlFlow::Break(()),
                }
            }
        }

        impl<T, E> Try for Result<T, E> {
            type Output = T;
            type Residual = E;

            fn from_output(output: Self::Output) -> Self {
                Ok(output)
            }

            fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
                match self {
                    Ok(x) => ControlFlow::Continue(x),
                    Err(e) => ControlFlow::Break(e),
                }
            }
        }

        impl<T, E> Try for Poll<Option<Result<T, E>>> {
            type Output = Poll<Option<T>>;
            type Residual = Result<std::convert::Infallible, E>;

            fn from_output(output: Self::Output) -> Self {
                output.map(|x| x.map(Ok))
            }

            fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
                match self {
                    Poll::Ready(Some(Ok(x))) => ControlFlow::Continue(Poll::Ready(Some(x))),
                    Poll::Ready(Some(Err(e))) => ControlFlow::Break(Err(e)),
                    Poll::Ready(None) => ControlFlow::Continue(Poll::Ready(None)),
                    Poll::Pending => ControlFlow::Continue(Poll::Pending),
                }
            }
        }

        impl<T, E> Try for Poll<Result<T, E>> {
            type Output = Poll<T>;
            type Residual = Result<std::convert::Infallible, E>;

            fn from_output(c: Self::Output) -> Self {
                c.map(Ok)
            }

            fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
                match self {
                    Poll::Ready(Ok(x)) => ControlFlow::Continue(Poll::Ready(x)),
                    Poll::Ready(Err(e)) => ControlFlow::Break(Err(e)),
                    Poll::Pending => ControlFlow::Continue(Poll::Pending),
                }
            }
        }
    }
}

pub use visit_rs_derive::*;

pub trait Visit<Visitor> {
    type Result;
    fn visit(&self, visitor: &mut Visitor) -> Self::Result;
}

pub trait VisitFields<Visitor> {
    type Result;
    fn visit_fields(&self, visitor: &mut Visitor) -> Self::Result;
}

pub trait VisitFieldsNamed<Visitor> {
    type Result;
    fn visit_fields_named(&self, visitor: &mut Visitor) -> Self::Result;
}

pub trait VisitMut<Visitor> {
    type Result;
    fn visit_mut(&mut self, visitor: &mut Visitor) -> Self::Result;
}

pub trait VisitMutFields<Visitor> {
    type Result;
    fn visit_mut_fields(&mut self, visitor: &mut Visitor) -> Self::Result;
}

pub trait VisitMutFieldsNamed<Visitor> {
    type Result;
    fn visit_mut_fields_named(&mut self, visitor: &mut Visitor) -> Self::Result;
}

pub trait AsyncVisit<Visitor> {
    type Result;
    fn visit_async<'a>(
        &'a self,
        visitor: &'a mut Visitor,
    ) -> impl Future<Output = Self::Result> + Send + 'a;
}

pub trait AsyncVisitFields<Visitor> {
    type Result;
    fn visit_fields_async<'a>(
        &'a self,
        visitor: &'a mut Visitor,
    ) -> impl Future<Output = Self::Result> + Send + 'a;
}

pub trait AsyncVisitFieldsNamed<Visitor> {
    type Result;
    fn visit_fields_named_async<'a>(
        &'a self,
        visitor: &'a mut Visitor,
    ) -> impl Future<Output = Self::Result> + Send + 'a;
}

pub trait ParallelAsyncVisitFields<Visitor> {
    type Result;
    fn parallel_visit_fields_async<'a>(
        &'a self,
        visitor: &Visitor,
    ) -> impl Future<Output = Self::Result> + Send + 'a
    where
        Visitor: 'a;
}

pub trait ParallelAsyncVisitFieldsNamed<Visitor> {
    type Result;
    fn parallel_visit_fields_named_async<'a>(
        &'a self,
        visitor: &Visitor,
    ) -> impl Future<Output = Self::Result> + Send + 'a
    where
        Visitor: 'a;
}

pub trait AsyncVisitMut<Visitor> {
    type Result;
    fn visit_mut_async<'a>(
        &'a mut self,
        visitor: &'a mut Visitor,
    ) -> impl Future<Output = Self::Result> + Send + 'a;
}

pub trait AsyncVisitMutFields<Visitor> {
    type Result;
    fn visit_mut_fields_async<'a>(
        &'a mut self,
        visitor: &'a mut Visitor,
    ) -> impl Future<Output = Self::Result> + Send + 'a;
}

pub trait AsyncVisitMutFieldsNamed<Visitor> {
    type Result;
    fn visit_mut_fields_named_async<'a>(
        &'a mut self,
        visitor: &'a mut Visitor,
    ) -> impl Future<Output = Self::Result> + Send + 'a;
}

pub struct Named<'a, T> {
    pub name: Option<&'static str>,
    pub value: &'a T,
}

pub struct NamedMut<'a, T> {
    pub name: Option<&'static str>,
    pub value: &'a mut T,
}
