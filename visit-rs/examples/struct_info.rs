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
    println!("  IS_NAMED: {}", NamedStruct::IS_NAMED);
    println!("  FIELD_COUNT: {}", NamedStruct::FIELD_COUNT);

    println!("\nTupleStruct:");
    println!("  IS_NAMED: {}", TupleStruct::IS_NAMED);
    println!("  FIELD_COUNT: {}", TupleStruct::FIELD_COUNT);

    assert!(NamedStruct::IS_NAMED);
    assert_eq!(NamedStruct::FIELD_COUNT, 3);

    assert!(!TupleStruct::IS_NAMED);
    assert_eq!(TupleStruct::FIELD_COUNT, 3);

    println!("\nStructInfo trait works correctly!");
}
