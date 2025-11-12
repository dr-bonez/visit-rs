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

pub trait StructInfo {
    const NAME: &'static str;
    const NAMED_FIELDS: bool;
    const FIELD_COUNT: usize;
}

pub trait VisitFields<V: Visitor>: StructInfo {
    fn visit_fields<'a>(&'a self, visitor: &'a mut V) -> impl Iterator<Item = V::Result> + 'a;
}

pub trait VisitFieldsStatic<V: Visitor>: StructInfo {
    fn visit_fields_static<'a>(visitor: &'a mut V) -> impl Iterator<Item = V::Result> + 'a;
}

pub trait VisitFieldsAsync<V: Visitor>: StructInfo {
    fn visit_fields_async<'a>(
        &'a self,
        visitor: &'a mut V,
    ) -> impl Stream<Item = V::Result> + Send + 'a
    where
        V: Send,
        V::Result: Send;
}

pub trait VisitFieldsStaticAsync<V: Visitor>: StructInfo {
    fn visit_fields_static_async<'a>(visitor: &'a mut V) -> impl Stream<Item = V::Result> + 'a
    where
        V: Send,
        V::Result: Send;
}

pub trait VisitFieldsNamed<V: Visitor>: StructInfo {
    fn visit_fields_named<'a>(&'a self, visitor: &'a mut V)
        -> impl Iterator<Item = V::Result> + 'a;
}

pub trait VisitFieldsStaticNamed<V: Visitor>: StructInfo {
    fn visit_fields_static_named<'a>(visitor: &'a mut V) -> impl Iterator<Item = V::Result> + 'a;
}

pub trait VisitFieldsNamedAsync<V: Visitor>: StructInfo {
    fn visit_fields_named_async<'a>(
        &'a self,
        visitor: &'a mut V,
    ) -> impl Stream<Item = V::Result> + Send + 'a
    where
        V: Send,
        V::Result: Send;
}

pub trait VisitFieldsStaticNamedAsync<V: Visitor>: StructInfo {
    fn visit_fields_static_named_async<'a>(
        visitor: &'a mut V,
    ) -> impl Stream<Item = V::Result> + Send + 'a
    where
        V: Send,
        V::Result: Send;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Named<'a, T: ?Sized> {
    pub name: Option<&'static str>,
    pub value: &'a T,
}

pub struct Static<T: ?Sized> {
    _phantom: PhantomData<T>,
}
impl<T: ?Sized> std::fmt::Debug for Static<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>()).finish()
    }
}
unsafe impl<T: ?Sized> Send for Static<T> {}
unsafe impl<T: ?Sized> Sync for Static<T> {}
impl<T: ?Sized> Default for Static<T> {
    fn default() -> Self {
        Static::new()
    }
}
impl<T: ?Sized> Clone for Static<T> {
    fn clone(&self) -> Self {
        Self::new()
    }
}
impl<T: ?Sized> Copy for Static<T> {}
impl<T: ?Sized + 'static, U: ?Sized + 'static> PartialEq<Static<U>> for Static<T> {
    fn eq(&self, _: &Static<U>) -> bool {
        std::any::TypeId::of::<T>() == std::any::TypeId::of::<U>()
    }
}
impl<T: ?Sized + 'static> Eq for Static<T> {}

impl<T: ?Sized> Static<T> {
    pub const fn new() -> Self {
        Static {
            _phantom: PhantomData,
        }
    }
}
