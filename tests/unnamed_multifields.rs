#![allow(dead_code)]

use enum_utility_macros::generate_enum_helper;

#[generate_enum_helper(
    TagEnum, RefEnum, MutEnum, is, unwrap, get, to_tag, unwrap_ref, unwrap_mut, as_ref, as_mut,
    get_mut
)]
enum MyEnum {
    Int32(i32, i16, u8),
    Float(f32, f32, f64),
    String(String, &'static str, &'static [u8]),
}

#[test]
fn test_tag_enum() {
    let mut v1 = MyEnum::Int32(0, 1, 2);
    let mut v2 = MyEnum::Float(0.0, 1.0, -1.0);
    let mut v3 = MyEnum::String("".into(), "", b"");

    let t1 = MyEnumTag::Int32;
    let t2 = MyEnumTag::Float;
    let t3 = MyEnumTag::String;

    assert_eq!(t1, v1.to_tag());
    assert_eq!(t2, v2.to_tag());
    assert_eq!(t3, v3.to_tag());

    let _r1 = MyEnumRef::Int32(&0, &1, &2);
    let _r2 = MyEnumRef::Float(&0.0, &1.0, &-1.0);
    let _r3 = MyEnumRef::String(&String::new(), &"Hello", &(b"egwer".as_slice()));

    let _m1 = MyEnumMut::Int32(&mut 0, &mut 1, &mut 2);
    let _m2 = MyEnumMut::Float(&mut 0.0, &mut 1.0, &mut 2.0);
    let _m3 = MyEnumMut::String(&mut String::new(), &mut "Hello", &mut (b"owefo".as_slice()));

    assert!(v1.is_int32());
    assert!(!v1.is_float());
    assert!(!v1.is_string());

    let _: (&i32, &i16, &u8) = v1.unwrap_ref_int32();
    let _: (&f32, &f32, &f64) = v2.unwrap_ref_float();
    let _: (&String, &&str, &&[u8]) = v3.unwrap_ref_string();

    let _: (&mut i32, &mut i16, &mut u8) = v1.as_mut().get_int32().unwrap();
    let _: (&mut f32, &mut f32, &mut f64) = v2.as_mut().get_float().unwrap();
    let _: (&mut String, &mut &str, &mut &[u8]) = v3.as_mut().get_string().unwrap();
}
