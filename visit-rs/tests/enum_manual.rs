use visit_rs::{
    EnumInfo, EnumInfoData, Static, StructInfoData, Variant, Visit, VisitVariant,
    VisitVariantFields, VisitVariantFieldsStatic, VisitVariantsStatic, Visitor,
};

enum TestEnum {
    Unit,
    Single(String),
    Tuple(i32, f32),
    Struct { foo: String, bar: Vec<u8> },
}
impl EnumInfo for TestEnum {
    const DATA: EnumInfoData = EnumInfoData {
        name: "TestEnum",
        variant_count: 4,
    };
    fn variants() -> impl IntoIterator<Item = visit_rs::StructInfoData> + Send + Sync + 'static {
        [
            StructInfoData {
                name: "Unit",
                named_fields: false,
                field_count: 0,
            },
            StructInfoData {
                name: "Single",
                named_fields: false,
                field_count: 1,
            },
            StructInfoData {
                name: "Tuple",
                named_fields: false,
                field_count: 2,
            },
            StructInfoData {
                name: "Struct",
                named_fields: true,
                field_count: 2,
            },
        ]
    }
    fn variant_info(&self) -> StructInfoData {
        match self {
            Self::Unit => StructInfoData {
                name: "Unit",
                named_fields: false,
                field_count: 0,
            },
            Self::Single(_) => StructInfoData {
                name: "Single",
                named_fields: false,
                field_count: 1,
            },
            Self::Tuple(_, _) => StructInfoData {
                name: "Tuple",
                named_fields: false,
                field_count: 2,
            },
            Self::Struct { .. } => StructInfoData {
                name: "Struct",
                named_fields: true,
                field_count: 2,
            },
        }
    }
    fn variant_info_by_name(name: &str) -> Option<StructInfoData> {
        Some(match name {
            "Unit" => StructInfoData {
                name: "Unit",
                named_fields: false,
                field_count: 0,
            },
            "Single" => StructInfoData {
                name: "Single",
                named_fields: false,
                field_count: 1,
            },
            "Tuple" => StructInfoData {
                name: "Tuple",
                named_fields: false,
                field_count: 2,
            },
            "Struct" => StructInfoData {
                name: "Struct",
                named_fields: true,
                field_count: 2,
            },
            _ => return None,
        })
    }
}
impl<V> VisitVariant<V> for TestEnum
where
    V: Visitor,
    for<'a> Variant<'a, Self>: Visit<V>,
{
    fn visit_variant(&self, visitor: &mut V) -> V::Result {
        Variant {
            info: self.variant_info(),
            value: self,
        }
        .visit(visitor)
    }
}
impl<V> VisitVariantsStatic<V> for TestEnum
where
    V: Visitor,
    for<'a> Variant<'a, Static<Self>>: Visit<V>,
{
    fn visit_variants_static<'a>(visitor: &'a mut V) -> impl Iterator<Item = V::Result> + 'a {
        Self::variants().into_iter().map(|info| {
            Variant {
                info,
                value: Static::new_ref(),
            }
            .visit(visitor)
        })
    }
}
impl<V> VisitVariantFields<V> for TestEnum
where
    V: Visitor,
    String: Visit<V>,
    i32: Visit<V>,
    f32: Visit<V>,
    Vec<u8>: Visit<V>,
{
    fn visit_variant_fields<'a>(
        &'a self,
        visitor: &'a mut V,
    ) -> impl Iterator<Item = <V as Visitor>::Result> + 'a {
        let mut i = 0;
        std::iter::from_fn(move || {
            let res = match self {
                Self::Unit => match i {
                    _ => return None,
                },
                Self::Single(tup_0) => match i {
                    0 => tup_0.visit(visitor),
                    _ => return None,
                },
                Self::Tuple(tup_0, tup_1) => match i {
                    0 => tup_0.visit(visitor),
                    1 => tup_1.visit(visitor),
                    _ => return None,
                },
                Self::Struct { foo, bar } => match i {
                    0 => foo.visit(visitor),
                    1 => bar.visit(visitor),
                    _ => return None,
                },
            };
            i += 1;
            Some(res)
        })
    }
}

impl<V> VisitVariantFieldsStatic<V> for TestEnum
where
    V: Visitor,
    Static<String>: Visit<V>,
    Static<i32>: Visit<V>,
    Static<f32>: Visit<V>,
    Static<Vec<u8>>: Visit<V>,
{
    fn visit_variant_fields_static<'a>(
        info: &'a StructInfoData,
        visitor: &'a mut V,
    ) -> impl Iterator<Item = <V as Visitor>::Result> + 'a {
        let mut i = 0;
        std::iter::from_fn(move || {
            let res = match info.name {
                "Unit" => match i {
                    _ => return None,
                },
                "Single" => match i {
                    0 => Static::<String>::new().visit(visitor),
                    _ => return None,
                },
                "Tuple" => match i {
                    0 => Static::<i32>::new().visit(visitor),
                    1 => Static::<f32>::new().visit(visitor),
                    _ => return None,
                },
                "Struct" => match i {
                    0 => Static::<String>::new().visit(visitor),
                    1 => Static::<Vec<u8>>::new().visit(visitor),
                    _ => return None,
                },
                x => {
                    debug_assert!(false, "UNREACHABLE: unknown variant {x}");
                    return None;
                }
            };
            i += 1;
            Some(res)
        })
    }
}
