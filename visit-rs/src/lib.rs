use std::any::Any;
use std::marker::PhantomData;

use futures::Stream;

pub use visit_rs_derive::*;

#[cfg(feature = "serde")]
pub mod serde;

pub mod lib {
    pub use async_stream;
    pub use futures;
}

pub trait Visitor {
    type Result;
}

pub trait Visit<V: Visitor> {
    fn visit(&self, visitor: &mut V) -> V::Result;
}

pub trait VisitAsync<V: Visitor> {
    fn visit_async<'a>(&'a self, visitor: &'a mut V) -> impl Future<Output = V::Result> + Send + 'a
    where
        V: Send,
        V::Result: Send;
}

pub trait VisitFields<V: Visitor> {
    fn visit_fields<'a>(&'a self, visitor: &'a mut V) -> impl Iterator<Item = V::Result> + 'a;
}

pub trait VisitFieldsAsync<V: Visitor> {
    fn visit_fields_async<'a>(
        &'a self,
        visitor: &'a mut V,
    ) -> impl Stream<Item = V::Result> + Send + 'a
    where
        V: Send,
        V::Result: Send;
}

pub trait VisitFieldsNamed<V: Visitor> {
    fn visit_fields_named<'a>(&'a self, visitor: &'a mut V)
        -> impl Iterator<Item = V::Result> + 'a;
}

pub trait VisitFieldsNamedAsync<V: Visitor> {
    fn visit_fields_named_async<'a>(
        &'a self,
        visitor: &'a mut V,
    ) -> impl Stream<Item = V::Result> + Send + 'a
    where
        V: Send,
        V::Result: Send;
}

pub struct Named<'a, T> {
    pub name: Option<&'static str>,
    pub value: &'a T,
}

pub struct NamedMut<'a, T> {
    pub name: Option<&'static str>,
    pub value: &'a mut T,
}
