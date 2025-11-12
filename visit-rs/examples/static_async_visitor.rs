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

#[derive(VisitFields)]
#[visit(rename = "EmptyStructure")]
#[allow(unused)]
struct Empty {}

#[derive(VisitFields)]
#[visit(rename = "ServerSettings")]
struct Settings {
    timeout: i32,
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

    println!("\nTesting StructInfo:");
    println!("  Config::NAME = {}", Config::NAME);
    println!("  Config::NAMED_FIELDS = {}", Config::NAMED_FIELDS);
    println!("  Config::FIELD_COUNT = {}", Config::FIELD_COUNT);
    assert_eq!(Config::NAME, "Config");
    assert_eq!(Config::NAMED_FIELDS, true);
    assert_eq!(Config::FIELD_COUNT, 3);

    println!("\n  Empty::NAME = {}", Empty::NAME);
    println!("  Empty::NAMED_FIELDS = {}", Empty::NAMED_FIELDS);
    println!("  Empty::FIELD_COUNT = {}", Empty::FIELD_COUNT);
    assert_eq!(Empty::NAME, "EmptyStructure");
    assert_eq!(Empty::NAMED_FIELDS, true);
    assert_eq!(Empty::FIELD_COUNT, 0);

    println!("\n  Settings::NAME = {}", Settings::NAME);
    println!("  Settings::NAMED_FIELDS = {}", Settings::NAMED_FIELDS);
    println!("  Settings::FIELD_COUNT = {}", Settings::FIELD_COUNT);
    assert_eq!(Settings::NAME, "ServerSettings");
    assert_eq!(Settings::NAMED_FIELDS, true);
    assert_eq!(Settings::FIELD_COUNT, 1);

    println!("\nAll async static visitors and StructInfo work!");
}
