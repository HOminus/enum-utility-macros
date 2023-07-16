#![allow(dead_code)]

use enum_utility_macros::generate_enum_helper;

#[generate_enum_helper(
    TagEnum, RefEnum, MutEnum, is, unwrap, get, to_tag, unwrap_ref, unwrap_mut, as_ref, as_mut,
    get_mut
)]
enum MyEnum<T, G> {
    Int32(i32),
    Float(f32),
    Double(f64),
    Template(T),
    Generic(G),
}

#[test]
fn test_tag_enum() {
    let mut v1: MyEnum<String, &dyn Send> = MyEnum::Int32(0);
    let mut v2: MyEnum<String, &dyn Send> = MyEnum::Float(0.0);
    let mut v3: MyEnum<String, &dyn Send> = MyEnum::Double(0.0);

    let t1 = MyEnumTag::Int32;
    let t2 = MyEnumTag::Float;
    let t3 = MyEnumTag::Double;

    assert_eq!(t1, v1.to_tag());
    assert_eq!(t2, v2.to_tag());
    assert_eq!(t3, v3.to_tag());

    let _r1 = MyEnumRef::<String, &dyn Send>::Int32(&0);
    let _r2 = MyEnumRef::<String, &dyn Send>::Float(&0.0);
    let _r3 = MyEnumRef::<String, &dyn Send>::Double(&0.0);

    let _m1 = MyEnumMut::<String, &dyn Send>::Int32(&mut 0);
    let _m2 = MyEnumMut::<String, &dyn Send>::Float(&mut 0.0);
    let _m3 = MyEnumMut::<String, &dyn Send>::Double(&mut 0.0);

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
