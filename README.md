owning-bytes
============

[![Build Status](https://travis-ci.org/erikjohnston/rust-owning-bytes.svg?branch=master)](https://travis-ci.org/erikjohnston/rust-owning-bytes)

A library that allows passing around a parsed object that depends on an
underlying buffer.


[Documentation](https://erikjohnston.github.io/rust-owning-bytes/owning_bytes/index.html)


# Getting Started

Add the following to your Cargo.toml:

```toml
[dependencies.owning-bytes]
git = "https://github.com/erikjohnston/rust-owning-bytes.git"
```

# Example

```rust
extern crate owning_bytes;
use owning_bytes::OwningByteBuf;

struct ExampleParsed<'a> {
    payload: &'a [u8],
}

fn create_from_vec(vec: Vec<u8>) -> OwningByteBuf<ExampleParsed<'static>> {
    OwningByteBuf::from_vec(vec, |buf| ExampleParsed { payload: &buf[1..3] })
}

fn main() {
    let vec = vec![1, 2, 3, 4];

    let parsed = create_from_vec(vec);

    assert_eq!(&parsed.get().payload, &[2, 3]);
}
```
