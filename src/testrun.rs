#![feature(macro_rules)]

use std::io::{IoResult, File};

macro_rules! run {
    ( $val0:expr ; $($($resn_1:ident)+ -> $valn:expr);+ ) => {
        $val0$(.and_then(|$resn_1+| $valn))+
    }
}

fn runfile() -> IoResult<String> {
    run! ( File::open(&Path::new("/etc/passwd")); mut f -> f.read_to_string() )
}

fn main() {
    println!("{}", runfile());
}
