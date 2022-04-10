#[derive(Debug, Eq, PartialEq)]
pub struct Ascii(
    Vec<u8>
    );
impl Ascii {
}
pub struct NotAsciiError(pub Vec<u8>);
fn main() {
    let mut a: usize = 0;
    let ptr = &mut a as *mut usize;
    unsafe {
        *ptr.offset(3) = 0x7ffff72f484c;
    }
}
