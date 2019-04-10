pub const FOO: i32 = 10;
pub const BAR: &'static str = "hello world";
pub const ZOM: f32 = 3.14;

pub(crate) const DONT_EXPORT_CRATE: i32 = 20;
const DONT_EXPORT_PRIV: i32 = 30;

#[repr(C)]
struct Foo {
    x: [i32; FOO],
}

#[no_mangle]
pub extern "C" fn root(x: Foo) { }
