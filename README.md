owning-bytes
============

[![Build Status](https://travis-ci.org/erikjohnston/rust-owning-bytes.svg?branch=master)](https://travis-ci.org/erikjohnston/rust-owning-bytes)

A library that allows passing around a parsed object that depends on an
underlying buffer.

Currently requires nightly.


[Documentation](https://erikjohnston.github.io/rust-owning-bytes/owning_bytes/index.html)


# Getting Started

Add the following to your Cargo.toml:

```toml
[dependencies.owning-bytes]
git = "https://github.com/erikjohnston/rust-owning-bytes.git"
```

# Examples

A simple example with a simple construction function:

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


An example where construction can fail:

```rust
extern crate owning_bytes;

use owning_bytes::OwningByteBuf;
use std::str::{self, Utf8Error};


fn create_from_vec(vec: Vec<u8>) -> Result<OwningByteBuf<&'static str>, Utf8Error> {
    OwningByteBuf::from_vec_res(vec, str::from_utf8).map_err(|(err, _vec)| err)
}

fn main() {
    let vec = b"Hello".to_vec();

    let parsed = create_from_vec(vec).unwrap();

    assert_eq!(*parsed.get(), "Hello");
}
```
