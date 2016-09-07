#![feature(alloc_system)]
extern crate alloc_system;

extern crate owning_bytes;

use owning_bytes::OwningByteBuf;


struct ExampleParsed<'a> {
    payload1: &'a [u8],
    payload2: &'a [u8],
    _test: Vec<u8>,
}

impl<'a> ExampleParsed<'a> {
    fn parse_buf(buf: &'a [u8]) -> ExampleParsed<'a> {
        ExampleParsed {
            payload1: &buf[0..2],
            payload2: &buf[3..5],
            _test: vec![0,1,2,3,4,5,6],
        }
    }
}


// Represents a read from e.g. a TCP stream
fn read() -> Vec<u8> {
    vec![0, 1, 3, 4, 5, 6, 7]
}

// Example read + parse step
fn get_next_parsed() -> OwningByteBuf<ExampleParsed<'static>> {
    let vec = read();

    OwningByteBuf::from_vec(vec, ExampleParsed::parse_buf)
}


fn main() {
    let parsed = get_next_parsed();

    println!("Payload 1: {:?}", parsed.get().payload1);
    println!("Payload 1: {:?}", parsed.get().payload2);

    let underlying_vec = parsed.into_vec();

    println!("From vec {:?}", underlying_vec);

    get_next_parsed();
}
