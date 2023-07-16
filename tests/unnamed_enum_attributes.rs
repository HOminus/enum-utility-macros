#![allow(dead_code)]

use std::fmt::Debug;

use enum_utility_macros::generate_enum_helper;

#[generate_enum_helper(
    TagEnum, RefEnum, MutEnum, is, unwrap, get, to_tag, unwrap_ref, unwrap_mut, as_ref, as_mut,
    get_mut
)]
#[derive(Clone, Debug, PartialEq)]
enum MyEnum {
    Int32(i32),
    Float(f32),
    Double(f64),
}

#[test]
fn unnamed_enum_attributes_test() {
    let mut m1 = MyEnum::Int32(1);
    let r1 = m1.as_ref();
    format!("{m1:?} {r1:?}");

    #[allow(unused_assignments)]
    let mut m2 = m1.clone();

    m2 = MyEnum::Int32(5);
    assert_ne!(m1, m2);

    let g1 = m1.as_mut();
    format!("{g1:?}");
    let MyEnumMut::Int32(i) = g1 else {
        panic!()
    };

    *i = 5;
    assert_eq!(m1, m2);
}
