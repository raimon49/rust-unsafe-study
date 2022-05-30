#[derive(Debug, Eq, PartialEq)]
pub struct Ascii(
    Vec<u8> // ASCIIテキストだけを保持する 0 - 0x7f までのバイト列
    );

impl Ascii {
    // 引数 bytes 内のASCIIテキストから型 Ascii を作る
    // ASCIIでない文字列が入っていたらNotAsciiErrorを返す
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Ascii, NotAsciiError> {
        if bytes.iter().any(|&_byte| !bytes.is_ascii()) {
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

fn very_trustworthy(shared: &i32) {
    unsafe {
        // 引数で受け取った共有ポインタを可変ポインタに変換し、書き換えている（未定義動作）
        let mutable = shared as *const i32 as *mut i32;
        *mutable = 20;
    }
}

fn option_to_raw<T>(opt: Option<&T>) -> *const T {
    match opt {
        None => std::ptr::null(),
        Some(r) => r as *const T
    }
}

fn distance<T>(left: *const T, right: *const T) -> isize {
    // 2つのrawポインタを仮引数で受け取り、両ポインタ間のメモリアドレスの距離を返す
    (left as isize - right as isize) / std::mem::size_of::<T>() as isize
}

mod ref_with_flag {
    use std::marker::PhantomData;
    use std::mem::align_of;

    // 古典的なbit操作をRustで安全にラップした型
    // 型Tは少なくとも2バイト単位でアライメントされているものでなければならない
    pub struct RefWithFlag<'a, T:'a> {
        ptr_and_bit: usize,
        behaves_like: PhantomData<&'a T>
    }

    impl<'a, T:'a> RefWithFlag<'a, T> {
        pub fn new(ptr: &'a T, flag: bool) -> RefWithFlag<T> {
            assert!(align_of:: <T>() % 2 == 0); // 最下位ビットがゼロであるか検証してからrawポインタに変換
            RefWithFlag {
                // 参照->rawポインタ->usizeに変換（usizeはどんな計算機でもポインタ型を保持するのに十分なサイズ）
                ptr_and_bit: ptr as *const T as usize | flag as usize,
                // メモリを消費しないゼロサイズの型（生存期間をどう扱うかRustコンパイラに教えるために必要なフィールドで、これが無いとコンパイルできない）
                behaves_like: PhantomData
            }
        }

        pub fn get_ref(&self) -> &'a T {
            unsafe {
                let ptr = (self.ptr_and_bit & !1) as *const T;
                &*ptr
            }
        }

        pub fn get_flag(&self) -> bool {
            // 最下位ビットをマスクしてゼロかを返す
            self.ptr_and_bit & 1 != 0
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

    let illegal_bytes = vec![0xf7, 0xbf, 0xbf, 0xbf];
    let _illegal_ascii = unsafe {
        Ascii::from_bytes_unchecked(illegal_bytes);
    };

    // 無効なUTF8が入っている
    // let bogus: String = _illegal_ascii.into();
    // assert_eq!(bogus.chars().next().unwrap() as u32, 0x1ffffff);

    let i = 10;
    very_trustworthy(&i);
    println!("{}", i * 100); // 1000が期待値だが、very_trustworthy()の中で書き換えられて2000になる

    let mut x = 10;
    let ptr_x = &mut x as *mut i32; // *mut T は T へのrawポインタで、参照先の変更を許す

    let y = Box::new(20);
    let ptr_y = &*y as *const i32;  // *const T は T へのrawポインタで、参照先の読み出しのみを許す

    unsafe {
        *ptr_x += *ptr_y;
    }

    assert_eq!(x, 30);

    // 関数option_to_raw()の呼び出しにはunsafeブロックが登場しない rawポインタの参照解決だけがunsafe
    assert!(!option_to_raw(Some(&("pea", "pod"))).is_null());
    assert_eq!(option_to_raw::<i32>(None), std::ptr::null());

    // 先頭の要素と最後の要素のポインタ距離をrawポインタを渡して計算させる
    let trucks = vec!["garbage truck", "dump truck", "moonstruck"];
    let first = &trucks[0];
    let last = &trucks[2];
    assert_eq!(distance(last, first), 2);
    assert_eq!(distance(first, last), -2);

    // &vec![42_u8] as *const String; // casting `&std::vec::Vec<u8>` as `*const std::string::String` is invalid
    &vec![42_u8] as *const Vec<u8> as *const String; // この変換は許される

    let vec = vec![10, 20, 30];
    let flagged = ref_with_flag::RefWithFlag::new(&vec, true);
    assert_eq!(flagged.get_ref()[1], 20);
    assert_eq!(flagged.get_flag(), true);

    // 計算機プロセッサによって型のサイズとアライメントが決定される
    assert_eq!(std::mem::size_of::<i64>(), 8);
    assert_eq!(std::mem::align_of::<(i32, i32)>(), 4);

    let slice: &[i32] = &[1, 3, 9, 27, 81];
    assert_eq!(std::mem::size_of_val(slice), 20);
}
