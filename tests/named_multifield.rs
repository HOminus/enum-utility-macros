#![allow(dead_code)]

use enum_utility_macros::generate_enum_helper;

#[generate_enum_helper(
    TagEnum, RefEnum, MutEnum, is, unwrap, get, to_tag, unwrap_ref, unwrap_mut, as_ref, as_mut,
    get_mut
)]
enum MyEnum {
    Int32 {
        int1: i32,
        int2: i16,
        int3: u8,
    },
    Float {
        float: f32,
        double: f64,
    },
    String {
        string: String,
        slice: &'static str,
        bytes: &'static [u8],
    },
}

#[test]
fn test_tag_enum() {
    let mut v1 = MyEnum::Int32 {
        int1: 0,
        int2: 1,
        int3: 2,
    };
    let mut v2 = MyEnum::Float {
        float: 0.0,
        double: 1.0,
    };
    let mut v3 = MyEnum::String {
        string: "hello".into(),
        slice: "world",
        bytes: b"!".as_slice(),
    };

    let t1 = MyEnumTag::Int32;
    let t2 = MyEnumTag::Float;
    let t3 = MyEnumTag::String;

    assert_eq!(t1, v1.to_tag());
    assert_eq!(t2, v2.to_tag());
    assert_eq!(t3, v3.to_tag());

    let _r1 = MyEnumRef::Int32{int1: &0, int2: &1, int3: &3};
    let _r2 = MyEnumRef::Float{float: &0.0, double: &1.0};
    let _r3 = MyEnumRef::String{string: &String::new(), slice: &"Hello", bytes: &(b"joh".as_slice())};

    let _r1 = MyEnumRef::Int32{int1: &mut 0, int2: &mut 1, int3: &mut 3};
    let _r2 = MyEnumRef::Float{float: &mut 0.0, double: &mut 1.0};
    let _r3 = MyEnumRef::String{string: &mut String::new(), slice: &mut "Hello", bytes: &mut (b"joh".as_slice())};

    assert!(v1.is_int32());
    assert!(!v1.is_float());
    assert!(!v1.is_string());

    let _: (&i32, &i16, &u8) = v1.unwrap_ref_int32();
    let _: (&f32, &f64) = v2.unwrap_ref_float();
    let _: (&String, &&str, &&[u8]) = v3.unwrap_ref_string();

    let _: (&mut i32, &mut i16, &mut u8) = v1.as_mut().get_int32().unwrap();
    let _: (&mut f32, &mut f64) = v2.as_mut().get_float().unwrap();
    let _: (&mut String, &mut &str, &mut &[u8]) = v3.as_mut().get_string().unwrap();
}
