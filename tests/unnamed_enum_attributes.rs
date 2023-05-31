use enum_utility_macros::generate_enum_helper;

#[generate_enum_helper(TagEnum, RefEnum, MutEnum,
    is, unwrap, get, to_tag, unwrap_ref, unwrap_mut, as_ref, as_mut, get_mut)]
#[derive(Clone, Copy, Debug)]
enum MyEnum {
    Int32(i32),
    Float(f32),
    Double(f64),
}





#[test]
fn unnamed_enum_attributes_test() {
    let mut m1 = MyEnum::Int32(1);
    let r1 = m1.as_ref();
    println!("{m1:?} {r1:?}");

    let g1 = m1.as_mut();
    println!("{g1:?}");
}
