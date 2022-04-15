#[derive(Debug, Eq, PartialEq)]
pub struct Ascii(
    Vec<u8>
    );
impl Ascii {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Ascii, NotAsciiError> {
        if bytes.iter().any(|&byte| !bytes.is_ascii()) {
            return Err(NotAsciiError(bytes));
        }

        Ok(Ascii(bytes))
    }
}
#[derive(Debug, Eq, PartialEq)]
pub struct NotAsciiError(pub Vec<u8>);
impl From<Ascii> for String {
    fn from(ascii: Ascii) -> String {
        unsafe {
            String::from_utf8_unchecked(ascii.0)
        }
    }
}
fn main() {
    let mut a: usize = 0;
    let ptr = &mut a as *mut usize;
    unsafe {
        *ptr.offset(3) = 0x7ffff72f484c;
    }
}
