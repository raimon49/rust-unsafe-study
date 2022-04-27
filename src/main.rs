#[derive(Debug, Eq, PartialEq)]
pub struct Ascii(
    Vec<u8> // ASCIIテキストだけを保持する 0 - 0x7f までのバイト列
    );

impl Ascii {
    // 引数 bytes 内のASCIIテキストから型 Ascii を作る
    // ASCIIでない文字列が入っていたらNotAsciiErrorを返す
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Ascii, NotAsciiError> {
        if bytes.iter().any(|&byte| !bytes.is_ascii()) {
            return Err(NotAsciiError(bytes));
        }

        Ok(Ascii(bytes))
    }

    // 引数をチェックしないコンストラクタ
    // 呼び出し元は0x7f以下のバイトのみ引数に渡さないと未定義動作となるためunsafeキーワードでマーク
    pub unsafe fn from_bytes_unchecked(bytes: Vec<u8>) -> Ascii {
        Ascii(bytes)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct NotAsciiError(pub Vec<u8>);

impl From<Ascii> for String {
    fn from(ascii: Ascii) -> String {
        unsafe {
            // unsafeだが安全で効率的な変換
            // well-formedなASCIIテキストはwell-formedなUTF8テキストであるのは自明なため
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

    // ASCIIだけで構成されたバイトベクタ
    let bytes: Vec<u8> = b"ASCII and ye shall receive".to_vec();
    // ヒープの確保もテキストのコピーも行われない呼び出し
    let ascii: Ascii = Ascii::from_bytes(bytes)
        .unwrap();
    // unsafeで実装されておりゼロコストで変換できる
    let string = String::from(ascii);
    assert_eq!(string, "ASCII and ye shall receive");
}
