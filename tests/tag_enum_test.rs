use enum_utility_macros::generate_enum_helper;

#[generate_enum_helper(TagEnum, RefEnum, MutEnum,
    is, unwrap, get, to_tag, unwrap_ref, unwrap_mut, as_ref, as_mut, get_mut)]
enum MyEnum {
    Int32 {
        name: i32
    },
    Float(f32),
    Double(f64),
}

#[test]
fn test_tag_enum() {
    let mut v1 = MyEnum::Int32 {name: 0};
    let mut v2 = MyEnum::Float(0.0);
    let mut v3 = MyEnum::Double(0.0);

    let _t1 = MyEnumTag::Int32;
    let _t2 = MyEnumTag::Float;
    let _t3 = MyEnumTag::Double;

    let _r1 = MyEnumRef::Int32{name: &0};
    let _r2 = MyEnumRef::Float(&0.0);
    let _r3 = MyEnumRef::Double(&0.0);

    let mut m1 = MyEnumMut::Int32{name: &mut 0};
    let _m2 = MyEnumMut::Float(&mut 0.0);
    let _m3 = MyEnumMut::Double(&mut 0.0);

    let _: &mut i32 = m1.unwrap_int32();

    assert!(v1.is_int32());
    assert!(!v1.is_double());
    assert!(!v1.is_float());

    let _: &i32 = v1.unwrap_ref_int32();
    let _: &f32 = v2.unwrap_ref_float();
    let _: &f64 = v3.unwrap_ref_double();

    let _: &mut i32 = v1.as_mut().get_int32().unwrap();
    let _: &mut f32 = v2.as_mut().get_float().unwrap();
    let _: &mut f64 = v3.as_mut().get_double().unwrap();
}
