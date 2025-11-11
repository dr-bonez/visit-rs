use futures::StreamExt;
use visit_rs::*;

struct AsyncTypeVisitor;

impl Visitor for AsyncTypeVisitor {
    type Result = String;
}

impl VisitAsync<AsyncTypeVisitor> for Static<String> {
    async fn visit_async(&self, _visitor: &mut AsyncTypeVisitor) -> String {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        "String".to_string()
    }
}

impl VisitAsync<AsyncTypeVisitor> for Static<i32> {
    async fn visit_async(&self, _visitor: &mut AsyncTypeVisitor) -> String {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        "i32".to_string()
    }
}

impl VisitAsync<AsyncTypeVisitor> for Static<bool> {
    async fn visit_async(&self, _visitor: &mut AsyncTypeVisitor) -> String {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        "bool".to_string()
    }
}

impl VisitAsync<AsyncTypeVisitor> for NamedStatic<String> {
    async fn visit_async(&self, _visitor: &mut AsyncTypeVisitor) -> String {
        format!("Named({}, String)", self.name.unwrap_or("UNNAMED"))
    }
}

impl VisitAsync<AsyncTypeVisitor> for NamedStatic<i32> {
    async fn visit_async(&self, _visitor: &mut AsyncTypeVisitor) -> String {
        format!("Named({}, i32)", self.name.unwrap_or("UNNAMED"))
    }
}

impl VisitAsync<AsyncTypeVisitor> for NamedStatic<bool> {
    async fn visit_async(&self, _visitor: &mut AsyncTypeVisitor) -> String {
        format!("Named({}, bool)", self.name.unwrap_or("UNNAMED"))
    }
}

#[derive(VisitFields)]
struct Config {
    host: String,
    port: i32,
    enabled: bool,
}

#[tokio::main]
async fn main() {
    let mut visitor = AsyncTypeVisitor;

    println!("Testing VisitFieldsStaticAsync:");
    let field_types: Vec<_> = Config::visit_fields_static_async(&mut visitor)
        .collect()
        .await;
    for (i, ty) in field_types.iter().enumerate() {
        println!("  Field {}: {}", i, ty);
    }
    assert_eq!(field_types, vec!["String", "i32", "bool"]);

    println!("\nTesting VisitFieldsStaticNamedAsync:");
    let field_types_named: Vec<_> = Config::visit_fields_static_named_async(&mut visitor)
        .collect()
        .await;
    for (i, ty) in field_types_named.iter().enumerate() {
        println!("  Field {}: {}", i, ty);
    }
    assert_eq!(
        field_types_named,
        vec![
            "Named(host, String)",
            "Named(port, i32)",
            "Named(enabled, bool)"
        ]
    );

    println!("\nAll async static visitors work!");
}
