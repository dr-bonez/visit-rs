use visit_rs::{Visit, VisitFields};

struct SizeVisitor {
    size: usize,
}

impl Visit<SizeVisitor> for String {
    type Result = ();

    fn visit(&self, visitor: &mut SizeVisitor) -> Self::Result {
        visitor.size += self.len();
    }
}
impl<T: Visit<SizeVisitor>> Visit<SizeVisitor> for Vec<T> {
    type Result = ();

    fn visit(&self, visitor: &mut SizeVisitor) -> Self::Result {
        for item in self {
            item.visit(visitor);
        }
    }
}

macro_rules! impl_size_visit_primitive {
    ($($t:ty),*) => {
        $(
            impl Visit<SizeVisitor> for $t {
                type Result = ();

                fn visit(&self, visitor: &mut SizeVisitor) -> Self::Result {
                    visitor.size += std::mem::size_of::<$t>();
                }
            }
        )*
    };
}
impl_size_visit_primitive!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, bool, char);

// #[derive(VisitFields)]
struct MyStruct {
    a: u32,
    b: String,
    c: Vec<u8>,
}
impl<Visitor> visit_rs::VisitFields<Visitor> for MyStruct
where
    u32: visit_rs::Visit<Visitor, Result = ()>,
    String: visit_rs::Visit<Visitor, Result = ()>,
    Vec<u8>: visit_rs::Visit<Visitor, Result = ()>,
{
    type Result = ();
    fn visit_fields(&self, visitor: &mut Visitor) -> Self::Result {
        visit_rs::Visit::<Visitor>::visit(&self.a, visitor);
        visit_rs::Visit::<Visitor>::visit(&self.b, visitor);
        visit_rs::Visit::<Visitor>::visit(&self.c, visitor);
        ()
    }
}
impl<Visitor> visit_rs::VisitFieldsNamed<Visitor> for MyStruct
where
    for<'__visit_rs__named> visit_rs::Named<'__visit_rs__named, u32>:
        visit_rs::Visit<Visitor, Result = ()>,
    for<'__visit_rs__named> visit_rs::Named<'__visit_rs__named, String>:
        visit_rs::Visit<Visitor, Result = ()>,
    for<'__visit_rs__named> visit_rs::Named<'__visit_rs__named, Vec<u8>>:
        visit_rs::Visit<Visitor, Result = ()>,
{
    type Result = ();
    fn visit_fields_named(&self, visitor: &mut Visitor) -> Self::Result {
        visit_rs::Visit::<Visitor>::visit(
            &visit_rs::Named {
                name: Some(stringify!(a)),
                value: &self.a,
            },
            visitor,
        );
        visit_rs::Visit::<Visitor>::visit(
            &visit_rs::Named {
                name: Some(stringify!(b)),
                value: &self.b,
            },
            visitor,
        );
        visit_rs::Visit::<Visitor>::visit(
            &visit_rs::Named {
                name: Some(stringify!(c)),
                value: &self.c,
            },
            visitor,
        );
        ()
    }
}
impl<Visitor> visit_rs::VisitMutFields<Visitor> for MyStruct
where
    u32: visit_rs::VisitMut<Visitor, Result = ()>,
    String: visit_rs::VisitMut<Visitor, Result = ()>,
    Vec<u8>: visit_rs::VisitMut<Visitor, Result = ()>,
{
    type Result = ();
    fn visit_mut_fields(&mut self, visitor: &mut Visitor) -> Self::Result {
        visit_rs::VisitMut::<Visitor>::visit_mut(&mut self.a, visitor);
        visit_rs::VisitMut::<Visitor>::visit_mut(&mut self.b, visitor);
        visit_rs::VisitMut::<Visitor>::visit_mut(&mut self.c, visitor);
        ()
    }
}
impl<Visitor> visit_rs::VisitMutFieldsNamed<Visitor> for MyStruct
where
    for<'__visit_rs__named> visit_rs::NamedMut<'__visit_rs__named, u32>:
        visit_rs::VisitMut<Visitor, Result = ()>,
    for<'__visit_rs__named> visit_rs::NamedMut<'__visit_rs__named, String>:
        visit_rs::VisitMut<Visitor, Result = ()>,
    for<'__visit_rs__named> visit_rs::NamedMut<'__visit_rs__named, Vec<u8>>:
        visit_rs::VisitMut<Visitor, Result = ()>,
{
    type Result = ();
    fn visit_mut_fields_named(&mut self, visitor: &mut Visitor) -> Self::Result {
        visit_rs::VisitMut::<Visitor>::visit_mut(
            &mut visit_rs::NamedMut {
                name: Some(stringify!(a)),
                value: &mut self.a,
            },
            visitor,
        );
        visit_rs::VisitMut::<Visitor>::visit_mut(
            &mut visit_rs::NamedMut {
                name: Some(stringify!(b)),
                value: &mut self.b,
            },
            visitor,
        );
        visit_rs::VisitMut::<Visitor>::visit_mut(
            &mut visit_rs::NamedMut {
                name: Some(stringify!(c)),
                value: &mut self.c,
            },
            visitor,
        );
        ()
    }
}
impl<Visitor> visit_rs::AsyncVisitFields<Visitor> for MyStruct
where
    Visitor: Send,
    u32: visit_rs::AsyncVisit<Visitor, Result = ()>,
    String: visit_rs::AsyncVisit<Visitor, Result = ()>,
    Vec<u8>: visit_rs::AsyncVisit<Visitor, Result = ()>,
{
    type Result = ();
    async fn visit_fields_async(&self, visitor: &mut Visitor) -> Self::Result {
        visit_rs::AsyncVisit::visit_async(&self.a, visitor).await;
        visit_rs::AsyncVisit::visit_async(&self.b, visitor).await;
        visit_rs::AsyncVisit::visit_async(&self.c, visitor).await;
        ()
    }
}
impl<Visitor> visit_rs::AsyncVisitFieldsNamed<Visitor> for MyStruct
where
    Visitor: Send,
    for<'__visit_rs__named> visit_rs::Named<'__visit_rs__named, u32>:
        visit_rs::AsyncVisit<Visitor, Result = ()>,
    for<'__visit_rs__named> visit_rs::Named<'__visit_rs__named, String>:
        visit_rs::AsyncVisit<Visitor, Result = ()>,
    for<'__visit_rs__named> visit_rs::Named<'__visit_rs__named, Vec<u8>>:
        visit_rs::AsyncVisit<Visitor, Result = ()>,
{
    type Result = ();
    async fn visit_fields_named_async(&self, visitor: &mut Visitor) -> Self::Result {
        visit_rs::AsyncVisit::visit_async(
            &visit_rs::Named {
                name: Some(stringify!(a)),
                value: &self.a,
            },
            visitor,
        )
        .await;
        visit_rs::AsyncVisit::visit_async(
            &visit_rs::Named {
                name: Some(stringify!(b)),
                value: &self.b,
            },
            visitor,
        )
        .await;
        visit_rs::AsyncVisit::visit_async(
            &visit_rs::Named {
                name: Some(stringify!(c)),
                value: &self.c,
            },
            visitor,
        )
        .await;
        ()
    }
}
impl<Visitor> visit_rs::ParallelAsyncVisitFields<Visitor> for MyStruct
where
    Visitor: Send + Clone,
    u32: visit_rs::AsyncVisit<Visitor, Result = ()>,
    String: visit_rs::AsyncVisit<Visitor, Result = ()>,
    Vec<u8>: visit_rs::AsyncVisit<Visitor, Result = ()>,
{
    type Result = ();
    fn parallel_visit_fields_async<'__visit_rs__a>(
        &'__visit_rs__a self,
        visitor: &Visitor,
    ) -> impl std::future::Future<Output = Self::Result> + Send + '__visit_rs__a
    where
        Visitor: '__visit_rs__a,
    {
        let mut futures = visit_rs::lib::futures::stream::FuturesOrdered::new();
        futures.push_back(visit_rs::lib::futures::FutureExt::boxed({
            let mut visitor = visitor.clone();
            async move { visit_rs::AsyncVisit::visit_async(&self.a, &mut visitor).await }
        }));
        futures.push_back(visit_rs::lib::futures::FutureExt::boxed({
            let mut visitor = visitor.clone();
            async move { visit_rs::AsyncVisit::visit_async(&self.b, &mut visitor).await }
        }));
        futures.push_back(visit_rs::lib::futures::FutureExt::boxed({
            let mut visitor = visitor.clone();
            async move { visit_rs::AsyncVisit::visit_async(&self.c, &mut visitor).await }
        }));
        async move {
            while let Some(res) =
                visit_rs::lib::futures::stream::StreamExt::next(&mut futures).await
            {
                res;
            }
            ()
        }
    }
}
impl<Visitor> visit_rs::ParallelAsyncVisitFieldsNamed<Visitor> for MyStruct
where
    Visitor: Send + Clone,
    for<'__visit_rs__named> visit_rs::Named<'__visit_rs__named, u32>:
        visit_rs::AsyncVisit<Visitor, Result = ()>,
    for<'__visit_rs__named> visit_rs::Named<'__visit_rs__named, String>:
        visit_rs::AsyncVisit<Visitor, Result = ()>,
    for<'__visit_rs__named> visit_rs::Named<'__visit_rs__named, Vec<u8>>:
        visit_rs::AsyncVisit<Visitor, Result = ()>,
{
    type Result = ();
    fn parallel_visit_fields_named_async<'__visit_rs__a>(
        &'__visit_rs__a self,
        visitor: &Visitor,
    ) -> impl std::future::Future<Output = Self::Result> + Send + '__visit_rs__a
    where
        Visitor: '__visit_rs__a,
    {
        let mut futures = visit_rs::lib::futures::stream::FuturesOrdered::new();
        futures.push_back(visit_rs::lib::futures::FutureExt::boxed({
            let mut visitor = visitor.clone();
            async move {
                visit_rs::AsyncVisit::visit_async(
                    &visit_rs::Named {
                        name: Some(stringify!(a)),
                        value: &self.a,
                    },
                    &mut visitor,
                )
                .await
            }
        }));
        futures.push_back(visit_rs::lib::futures::FutureExt::boxed({
            let mut visitor = visitor.clone();
            async move {
                visit_rs::AsyncVisit::visit_async(
                    &visit_rs::Named {
                        name: Some(stringify!(b)),
                        value: &self.b,
                    },
                    &mut visitor,
                )
                .await
            }
        }));
        futures.push_back(visit_rs::lib::futures::FutureExt::boxed({
            let mut visitor = visitor.clone();
            async move {
                visit_rs::AsyncVisit::visit_async(
                    &visit_rs::Named {
                        name: Some(stringify!(c)),
                        value: &self.c,
                    },
                    &mut visitor,
                )
                .await
            }
        }));
        async move {
            while let Some(res) =
                visit_rs::lib::futures::stream::StreamExt::next(&mut futures).await
            {
                res;
            }
            ()
        }
    }
}
impl<Visitor> visit_rs::AsyncVisitMutFields<Visitor> for MyStruct
where
    Visitor: Send,
    u32: visit_rs::AsyncVisitMut<Visitor, Result = ()>,
    String: visit_rs::AsyncVisitMut<Visitor, Result = ()>,
    Vec<u8>: visit_rs::AsyncVisitMut<Visitor, Result = ()>,
{
    type Result = ();
    async fn visit_mut_fields_async(&mut self, visitor: &mut Visitor) -> Self::Result {
        visit_rs::AsyncVisitMut::visit_mut_async(&mut self.a, visitor).await;
        visit_rs::AsyncVisitMut::visit_mut_async(&mut self.b, visitor).await;
        visit_rs::AsyncVisitMut::visit_mut_async(&mut self.c, visitor).await;
        ()
    }
}
impl<Visitor> visit_rs::AsyncVisitMutFieldsNamed<Visitor> for MyStruct
where
    Visitor: Send,
    for<'__visit_rs__named> visit_rs::NamedMut<'__visit_rs__named, u32>:
        visit_rs::AsyncVisitMut<Visitor, Result = ()>,
    for<'__visit_rs__named> visit_rs::NamedMut<'__visit_rs__named, String>:
        visit_rs::AsyncVisitMut<Visitor, Result = ()>,
    for<'__visit_rs__named> visit_rs::NamedMut<'__visit_rs__named, Vec<u8>>:
        visit_rs::AsyncVisitMut<Visitor, Result = ()>,
{
    type Result = ();
    async fn visit_mut_fields_named_async(&mut self, visitor: &mut Visitor) -> Self::Result {
        visit_rs::AsyncVisitMut::visit_mut_async(
            &mut visit_rs::NamedMut {
                name: Some(stringify!(a)),
                value: &mut self.a,
            },
            visitor,
        )
        .await;
        visit_rs::AsyncVisitMut::visit_mut_async(
            &mut visit_rs::NamedMut {
                name: Some(stringify!(b)),
                value: &mut self.b,
            },
            visitor,
        )
        .await;
        visit_rs::AsyncVisitMut::visit_mut_async(
            &mut visit_rs::NamedMut {
                name: Some(stringify!(c)),
                value: &mut self.c,
            },
            visitor,
        )
        .await;
        ()
    }
}
