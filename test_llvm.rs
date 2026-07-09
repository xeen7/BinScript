use inkwell::context::Context;
fn main() {
    let context = Context::create();
    let module = context.create_module("test");
    let i32_ty = context.i32_type();
    let fn_ty = i32_ty.fn_type(&[], false);
    let f1 = module.add_function("foo", fn_ty, None);
    let f2 = module.add_function("foo", fn_ty, None);
    println!("f1: {:?}", f1.get_name().to_str());
    println!("f2: {:?}", f2.get_name().to_str());
}
