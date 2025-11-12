use visit_rs::*;

#[derive(VisitFields)]
struct DefaultName {
    field: String,
}

#[derive(VisitFields)]
#[visit(rename = "CustomName")]
struct RenamedWithVisit {
    field: String,
}

#[derive(VisitFields)]
#[visit(rename = "AnotherCustomName")]
struct AnotherRenamed {
    field1: i32,
    field2: bool,
}

fn main() {
    println!("Testing StructInfo NAME constant:");

    println!("\nDefaultName (no rename):");
    println!("  NAME: {}", DefaultName::NAME);
    println!("  NAMED_FIELDS: {}", DefaultName::NAMED_FIELDS);
    println!("  FIELD_COUNT: {}", DefaultName::FIELD_COUNT);
    assert_eq!(DefaultName::NAME, "DefaultName");
    assert_eq!(DefaultName::NAMED_FIELDS, true);
    assert_eq!(DefaultName::FIELD_COUNT, 1);

    println!("\nRenamedWithVisit:");
    println!("  NAME: {}", RenamedWithVisit::NAME);
    println!("  NAMED_FIELDS: {}", RenamedWithVisit::NAMED_FIELDS);
    println!("  FIELD_COUNT: {}", RenamedWithVisit::FIELD_COUNT);
    assert_eq!(RenamedWithVisit::NAME, "CustomName");
    assert_eq!(RenamedWithVisit::NAMED_FIELDS, true);
    assert_eq!(RenamedWithVisit::FIELD_COUNT, 1);

    println!("\nAnotherRenamed:");
    println!("  NAME: {}", AnotherRenamed::NAME);
    println!("  NAMED_FIELDS: {}", AnotherRenamed::NAMED_FIELDS);
    println!("  FIELD_COUNT: {}", AnotherRenamed::FIELD_COUNT);
    assert_eq!(AnotherRenamed::NAME, "AnotherCustomName");
    assert_eq!(AnotherRenamed::NAMED_FIELDS, true);
    assert_eq!(AnotherRenamed::FIELD_COUNT, 2);

    println!("\nAll rename tests passed!");
}
