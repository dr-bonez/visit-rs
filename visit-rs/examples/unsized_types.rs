use visit_rs::*;

struct TypeVisitor;

impl Visitor for TypeVisitor {
    type Result = String;
}

// Sized types
impl Visit<TypeVisitor> for Static<String> {
    fn visit(&self, _visitor: &mut TypeVisitor) -> String {
        "String (sized)".to_string()
    }
}

impl Visit<TypeVisitor> for Static<i32> {
    fn visit(&self, _visitor: &mut TypeVisitor) -> String {
        "i32 (sized)".to_string()
    }
}

// Unsized types
impl Visit<TypeVisitor> for Static<str> {
    fn visit(&self, _visitor: &mut TypeVisitor) -> String {
        "str (unsized)".to_string()
    }
}

impl Visit<TypeVisitor> for Static<[i32]> {
    fn visit(&self, _visitor: &mut TypeVisitor) -> String {
        "[i32] (unsized slice)".to_string()
    }
}

// Named versions
impl Visit<TypeVisitor> for NamedStatic<String> {
    fn visit(&self, _visitor: &mut TypeVisitor) -> String {
        format!("Named({}, String)", self.name.unwrap_or("UNNAMED"))
    }
}

impl Visit<TypeVisitor> for NamedStatic<str> {
    fn visit(&self, _visitor: &mut TypeVisitor) -> String {
        format!("Named({}, str unsized)", self.name.unwrap_or("UNNAMED"))
    }
}

impl Visit<TypeVisitor> for NamedStatic<[i32]> {
    fn visit(&self, _visitor: &mut TypeVisitor) -> String {
        format!("Named({}, [i32] unsized)", self.name.unwrap_or("UNNAMED"))
    }
}

fn main() {
    let mut visitor = TypeVisitor;

    println!("Testing Static<T> with sized types:");
    println!("  {}", Static::<String>::new().visit(&mut visitor));
    println!("  {}", Static::<i32>::new().visit(&mut visitor));

    println!("\nTesting Static<T> with unsized types:");
    println!("  {}", Static::<str>::new().visit(&mut visitor));
    println!("  {}", Static::<[i32]>::new().visit(&mut visitor));

    println!("\nTesting NamedStatic<T> with sized types:");
    println!("  {}", NamedStatic::<String>::new(Some("my_string")).visit(&mut visitor));

    println!("\nTesting NamedStatic<T> with unsized types:");
    println!("  {}", NamedStatic::<str>::new(Some("my_str")).visit(&mut visitor));
    println!("  {}", NamedStatic::<[i32]>::new(Some("my_slice")).visit(&mut visitor));

    println!("\nStatic and NamedStatic support both sized and unsized types!");
}
