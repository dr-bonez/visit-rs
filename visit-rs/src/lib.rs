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

pub trait VisitFieldsStatic<V: Visitor> {
    fn visit_fields_static<'a>(visitor: &'a mut V) -> impl Iterator<Item = V::Result> + 'a;
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

pub trait VisitFieldsStaticAsync<V: Visitor> {
    fn visit_fields_static_async<'a>(visitor: &'a mut V) -> impl Stream<Item = V::Result> + 'a
    where
        V: Send,
        V::Result: Send;
}

pub trait VisitFieldsNamed<V: Visitor> {
    fn visit_fields_named<'a>(&'a self, visitor: &'a mut V)
    -> impl Iterator<Item = V::Result> + 'a;
}

pub trait VisitFieldsStaticNamed<V: Visitor> {
    fn visit_fields_static_named<'a>(visitor: &'a mut V) -> impl Iterator<Item = V::Result> + 'a;
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

pub trait VisitFieldsStaticNamedAsync<V: Visitor> {
    fn visit_fields_static_named_async<'a>(
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

pub struct Static<T> {
    _phantom: PhantomData<T>,
}

unsafe impl<T> Send for Static<T> {}
unsafe impl<T> Sync for Static<T> {}

impl<T> Static<T> {
    pub fn new() -> Self {
        Static {
            _phantom: PhantomData,
        }
    }
}

pub struct NamedStatic<T> {
    pub name: Option<&'static str>,
    _phantom: PhantomData<T>,
}

unsafe impl<T> Send for NamedStatic<T> {}
unsafe impl<T> Sync for NamedStatic<T> {}

impl<T> NamedStatic<T> {
    pub fn new(name: Option<&'static str>) -> Self {
        NamedStatic {
            name,
            _phantom: PhantomData,
        }
    }
}
