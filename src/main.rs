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

mod gap {
    use std;
    use std::ops::Range;

    pub struct GapBuffer<T> {
        storage: Vec<T>,

        gap: Range<usize>
    }

    impl<T> GapBuffer<T> {
        pub fn new() -> GapBuffer<T> {
            GapBuffer {
                storage: Vec::new(),
                gap: 0..0
            }
        }

        pub fn capacity(&self) -> usize {
            self.storage.capacity()
        }

        pub fn len(&self) -> usize {
            self.capacity() - self.gap.len()
        }

        pub fn position(&self) -> usize {
            self.gap.start
        }

        unsafe fn space(&self, index: usize) -> *const T {
            self.storage.as_ptr().offset(index as isize)
        }

        unsafe fn space_mut(&mut self, index: usize) -> *mut T {
            self.storage.as_mut_ptr().offset(index as isize)
        }

        fn index_to_raw(&self, index: usize) -> usize {
            if index < self.gap.start {
                index
            } else {
                index + self.gap.len()
            }
        }

        pub fn get(&self, index: usize) -> Option<&T> {
            let raw = self.index_to_raw(index);
            if raw < self.capacity() {
                unsafe {
                    Some(&*self.space(raw))
                }
            } else {
                None
            }
        }

        pub fn set_position(&mut self, pos: usize) {
            if pos > self.len() {
                panic!("index {} out of range for GapBuffer", pos);
            }

            unsafe {
                let gap = self.gap.clone();
                if pos > gap.start {
                    let distance = pos - gap.start;
                    std::ptr::copy(self.space(gap.end),
                                   self.space_mut(gap.start),
                                   distance);
                } else if pos < gap.start {
                    let distance = gap.start - pos;
                    std::ptr::copy(self.space(pos),
                                   self.space_mut(gap.end - distance),
                                   distance);
                }

            self.gap = pos .. pos + gap.len();
            }
        }

        pub fn remove(&mut self) -> Option<T> {
            if self.gap.end == self.capacity() {
                return None;
            }

            let element = unsafe {
                std::ptr::read(self.space(self.gap.end))
            };
            self.gap.end += 1;
            Some(element)
        }

        fn enlarge_gap(&mut self) {
            let mut new_capacity = self.capacity() * 2;
            if new_capacity == 0 {
                new_capacity = 4;
            }

            let mut new = Vec::with_capacity(new_capacity);
            let after_gap = self.capacity() - self.gap.end;
            let new_gap = self.gap.start .. new.capacity() - after_gap;
            unsafe {
                std::ptr::copy_nonoverlapping(self.space(0),
                                           new.as_mut_ptr(),
                                           self.gap.start);
                let new_gap_end = new.as_mut_ptr().offset(new_gap.end as isize);
                std::ptr::copy_nonoverlapping(self.space(self.gap.end),
                                           new_gap_end,
                                           after_gap);
            }

            self.storage = new;
            self.gap = new_gap;
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

    assert_eq!(x, 30); // *mut i32型のptr_xを通してポインタの指す値が更新されている

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
    assert_eq!(flagged.get_ref()[1], 20); // ラップしたvec参照の要素を取り出す
    assert_eq!(flagged.get_flag(), true); // ラップしたvecのメモリに保存した値boolを取り出す

    // 計算機プロセッサによって型のサイズとアラインメントが決定される
    assert_eq!(std::mem::size_of::<i64>(), 8);
    assert_eq!(std::mem::align_of::<(i32, i32)>(), 4);

    let slice: &[i32] = &[1, 3, 9, 27, 81];
    assert_eq!(std::mem::size_of_val(slice), 20);
    let text: &str = "alligator";
    assert_eq!(std::mem::size_of_val(text), 9);

    // トレイトオブジェクトそのものではなく、トレイトオブジェクトが指す値のサイズ・アラインメントを返す
    use std::fmt::Display;
    let unremarkable: &dyn Display = &193_u8;
    let remarkable: &dyn Display = &0.0072973525664;
    assert_eq!(std::mem::size_of_val(unremarkable), 1);
    assert_eq!(std::mem::align_of_val(remarkable), 8);
    {
        let pot = "pasta".to_string();
        let _plate;
        _plate = pot; // 変数potのメモリアドレスは未初期化状態になる
    }
    {
        let mut noodles = vec!["udon".to_string()]; // noodles[0]のみメモリ確保された状態
        let soba = "soba".to_string();
        let _last; // 最終的に変数lastだけが所有権を持つ
        noodles.push(soba); // noodles[1]にメモリ確保され、変数sobaは未初期化状態になる
        _last = noodles.pop().unwrap(); // noodles[1]は未初期化状態になる
    }
}
