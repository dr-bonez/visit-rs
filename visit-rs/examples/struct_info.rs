use visit_rs::*;

#[derive(VisitFields)]
struct NamedStruct {
    field1: String,
    field2: i32,
    field3: bool,
}

#[derive(VisitFields)]
struct TupleStruct(String, i32, bool);

fn main() {
    println!("NamedStruct:");
    println!("  NAME: {}", NamedStruct::NAME);
    println!("  NAMED_FIELDS: {}", NamedStruct::NAMED_FIELDS);
    println!("  FIELD_COUNT: {}", NamedStruct::FIELD_COUNT);

    println!("\nTupleStruct:");
    println!("  NAME: {}", TupleStruct::NAME);
    println!("  NAMED_FIELDS: {}", TupleStruct::NAMED_FIELDS);
    println!("  FIELD_COUNT: {}", TupleStruct::FIELD_COUNT);

    assert_eq!(NamedStruct::NAME, "NamedStruct");
    assert!(NamedStruct::NAMED_FIELDS);
    assert_eq!(NamedStruct::FIELD_COUNT, 3);

    assert_eq!(TupleStruct::NAME, "TupleStruct");
    assert!(!TupleStruct::NAMED_FIELDS);
    assert_eq!(TupleStruct::FIELD_COUNT, 3);

    println!("\nStructInfo trait works correctly!");
}
