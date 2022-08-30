use bech32;
use std::str;

fn main() {
    let test_hrp: &str = "aaa";
    let test_vec = "you look beautiful today".as_bytes().to_vec();

    // Example encoding.
    let result = bech32::encode(test_hrp, &test_vec);
    print_string(&result);

    // Example decoding.
    let (hrp, data) = bech32::decode(result);
    print_string(&hrp);
    print_string(&data);
}

fn print_string(v: &Vec<u8>) {
   println!("{}", str::from_utf8(v).unwrap());
}