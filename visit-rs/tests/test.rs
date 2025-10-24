#![feature(iter_macro, coroutines)]

use std::io::Write;
use std::pin;

use futures::future::BoxFuture;
use futures::{FutureExt, SinkExt, Stream, StreamExt, TryStreamExt};
use tokio::io::AsyncWrite;
use visit_rs::{Visit, VisitAsync, VisitFields, VisitFieldsAsync, Visitor};

struct SizeVisitor {
    size: usize,
}
impl Visitor for SizeVisitor {
    type Result = ();
}

impl Visit<SizeVisitor> for String {
    fn visit(&self, visitor: &mut SizeVisitor) {
        visitor.size += self.len();
    }
}
impl<T: Visit<SizeVisitor>> Visit<SizeVisitor> for Vec<T> {
    fn visit(&self, visitor: &mut SizeVisitor) {
        for item in self {
            item.visit(visitor);
        }
    }
}

macro_rules! impl_size_visit_primitive {
    ($($t:ty),*) => {
        $(
            impl Visit<SizeVisitor> for $t {
                fn visit(&self, visitor: &mut SizeVisitor) {
                    visitor.size += std::mem::size_of::<$t>();
                }
            }
        )*
    };
}
impl_size_visit_primitive!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, bool, char);

struct WriteVisitor<W: Write> {
    writer: W,
}
impl<W: Write> Visitor for WriteVisitor<W> {
    type Result = Result<(), std::io::Error>;
}

impl<W: Write> Visit<WriteVisitor<W>> for String {
    fn visit(&self, visitor: &mut WriteVisitor<W>) -> Result<(), std::io::Error> {
        visitor.writer.write_all(self.as_bytes())
    }
}
impl<W: Write> Visit<WriteVisitor<W>> for Vec<u8> {
    fn visit(&self, visitor: &mut WriteVisitor<W>) -> Result<(), std::io::Error> {
        visitor.writer.write_all(self)
    }
}
macro_rules! impl_write_visit_num {
    ($($t:ty),*) => {
        $(
            impl<W: Write> Visit<WriteVisitor<W>> for $t {
                fn visit(&self, visitor: &mut WriteVisitor<W>) -> Result<(), std::io::Error> {
                    let bytes = self.to_le_bytes();
                    visitor.writer.write_all(&bytes)
                }
            }
        )*
    };
}
impl_write_visit_num!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

struct AsyncWriteVisitor<W: AsyncWrite> {
    writer: W,
}
impl<W: AsyncWrite> Visitor for AsyncWriteVisitor<W>
where
    W: AsyncWrite + Unpin + Send,
{
    type Result = Result<(), std::io::Error>;
}

impl<W: AsyncWrite> VisitAsync<AsyncWriteVisitor<W>> for String
where
    W: AsyncWrite + Unpin + Send,
{
    async fn visit_async(&self, visitor: &mut AsyncWriteVisitor<W>) -> Result<(), std::io::Error> {
        use tokio::io::AsyncWriteExt;
        visitor.writer.write_all(self.as_bytes()).await
    }
}
impl<W: AsyncWrite> VisitAsync<AsyncWriteVisitor<W>> for Vec<u8>
where
    W: AsyncWrite + Unpin + Send,
{
    async fn visit_async(&self, visitor: &mut AsyncWriteVisitor<W>) -> Result<(), std::io::Error> {
        use tokio::io::AsyncWriteExt;
        visitor.writer.write_all(self).await
    }
}
macro_rules! impl_async_write_visit_num {
    ($($t:ty),*) => {
        $(
            impl<W: AsyncWrite> VisitAsync<AsyncWriteVisitor<W>> for $t
            where
                W: AsyncWrite + Unpin + Send,
            {
                async fn visit_async(
                    &self,
                    visitor: &mut AsyncWriteVisitor<W>,
                ) -> Result<(), std::io::Error>
                {
                    use tokio::io::AsyncWriteExt;
                    let bytes = self.to_le_bytes();
                    visitor.writer.write_all(&bytes).await
                }
            }
        )*
    };
}
impl_async_write_visit_num!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

#[derive(VisitFields)]
struct MyStruct {
    a: u32,
    b: String,
    c: Vec<u8>,
}

impl<W: AsyncWrite + Unpin + Send> VisitAsync<AsyncWriteVisitor<W>> for MyStruct {
    async fn visit_async(&self, visitor: &mut AsyncWriteVisitor<W>) -> Result<(), std::io::Error> {
        self.visit_fields_async(visitor).try_collect().await
    }
}

#[tokio::test]
async fn test_serialize_my_struct_async() {
    let mut sink = Vec::new();
    let mut visitor = AsyncWriteVisitor { writer: &mut sink };
    let my_struct = MyStruct {
        a: 42,
        b: "Hello".to_string(),
        c: vec![1, 2, 3, 4, 5],
    };
    my_struct.visit_async(&mut visitor).await.unwrap();
    let expected: Vec<u8> = vec![
        42, 0, 0, 0, // u32 in little-endian
        b'H', b'e', b'l', b'l', b'o', // "Hello"
        1, 2, 3, 4, 5,
    ];
    assert_eq!(sink, expected);
}

#[derive(VisitFields)]
struct MyGenericStruct<A> {
    a: A,
    b: MyStruct,
}

impl<A: VisitAsync<AsyncWriteVisitor<W>> + Sync, W: AsyncWrite + Unpin + Send>
    VisitAsync<AsyncWriteVisitor<W>> for MyGenericStruct<A>
{
    async fn visit_async(&self, visitor: &mut AsyncWriteVisitor<W>) -> Result<(), std::io::Error> {
        self.visit_fields_async(visitor).try_collect().await
    }
}

#[tokio::test]
async fn test_serialize_my_generic_struct_async() {
    let mut sink = Vec::new();
    let mut visitor = AsyncWriteVisitor { writer: &mut sink };
    let my_struct = MyGenericStruct {
        a: 64_u8,
        b: MyStruct {
            a: 42,
            b: "Hello".to_string(),
            c: vec![1, 2, 3, 4, 5],
        },
    };
    my_struct.visit_async(&mut visitor).await.unwrap();
    let expected: Vec<u8> = vec![
        64, // u8 in little-endian
        42, 0, 0, 0, // u32 in little-endian
        b'H', b'e', b'l', b'l', b'o', // "Hello"
        1, 2, 3, 4, 5,
    ];
    assert_eq!(sink, expected);
}
