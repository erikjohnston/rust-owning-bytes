#![feature(unique, alloc, heap_api, question_mark)]
#![warn(missing_docs)]

//! # Owning bytes
//!
//! This crate allows you to wrap up and parse around structs that have members that reference a block
//! of bytes. This is especially useful for writing protocol parses where you want to do zero copy
//! parsing.
//!
//! For example:
//!
//! ```
//! use owning_bytes::OwningByteBuf;
//!
//! struct ExampleParsed<'a> {
//! payload: &'a [u8],
//! }
//!
//! impl<'a> ExampleParsed<'a> {
//! pub fn parse_buf(buf: &'a [u8]) -> ExampleParsed<'a> {
//! ExampleParsed { payload: &buf[1..3] }
//! }
//! }
//!
//! fn create_from_vec(vec: Vec<u8>) -> OwningByteBuf<ExampleParsed<'static>> {
//! OwningByteBuf::from_vec(vec, ExampleParsed::parse_buf)
//! }
//!
//! fn main() {
//! let vec = vec![1, 2, 3, 4];
//!
//! let parsed = create_from_vec(vec);
//!
//! assert_eq!(&parsed.get().payload, &[2, 3]);
//! }
//! ```
//!
//! There are also methods for dealing with constructors that may fail:
//!
//! ```
//! use owning_bytes::OwningByteBuf;
//! use std::str::{self, Utf8Error};
//!
//!
//! fn create_from_vec(vec: Vec<u8>) -> Result<OwningByteBuf<&'static str>, Utf8Error> {
//! OwningByteBuf::from_vec_res(vec, str::from_utf8).map_err(|(err, _vec)| err)
//! }
//!
//! fn main() {
//! let vec = b"Hello".to_vec();
//!
//! let parsed = create_from_vec(vec).unwrap();
//!
//! assert_eq!(*parsed.get(), "Hello");
//! }
//! ```
//!

extern crate alloc;

use std::mem;
use std::ptr::Unique;
use std::slice;
use alloc::heap;

use std::convert::AsRef;

/// A wrapper around an array of bytes and an object T that references those bytes.
pub struct OwningByteBuf<T> {
    resource: Unique<u8>,
    len: usize,
    cap: usize,
    inner: T,
}

impl<T> OwningByteBuf<T> {
    /// Creates an OwningByteBuf from a vector and a constructing function
    pub fn from_vec<'a, F>(mut buf: Vec<u8>, f: F) -> OwningByteBuf<T>
        where F: FnOnce(&'a [u8]) -> T
    {
        let res = unsafe {
            let ptr = buf.as_mut_ptr();
            let len = buf.len();
            let cap = buf.capacity();
            let inner = f(slice::from_raw_parts(ptr, len));

            OwningByteBuf {
                resource: Unique::new(ptr),
                len: len,
                cap: cap,
                inner: inner,
            }
        };
        mem::forget(buf);
        res
    }

    /// Creates an OwningByteBuf from a vector and a constructing function that may fail
    ///
    /// ```
    /// use std::str;
    /// use owning_bytes::OwningByteBuf;
    ///
    /// let vec = b"Hello World".to_vec();
    /// let string = OwningByteBuf::from_vec_res(vec, str::from_utf8).unwrap();
    /// assert_eq!(*string.get(), "Hello World");
    /// ```
    pub fn from_vec_res<'a, F, E>(mut buf: Vec<u8>, f: F) -> Result<OwningByteBuf<T>, (E, Vec<u8>)>
        where F: FnOnce(&'a [u8]) -> Result<T, E>
    {
        let res = unsafe {
            let ptr = buf.as_mut_ptr();
            let len = buf.len();
            let cap = buf.capacity();
            OwningByteBuf {
                resource: Unique::new(ptr),
                len: len,
                cap: cap,
                inner: match f(slice::from_raw_parts(ptr, len)) {
                    Ok(t) => t,
                    Err(e) => return Err((e, buf)),
                },
            }
        };
        mem::forget(buf);
        Ok(res)
    }

    /// Creates an OwningByteBuf from a boxed slice and a constructing function
    pub fn from_box<'a, F>(mut buf: Box<[u8]>, f: F) -> OwningByteBuf<T>
        where F: FnOnce(&'a [u8]) -> T
    {
        let res = unsafe {
            let ptr = buf.as_mut_ptr();
            let len = buf.len();
            OwningByteBuf {
                resource: Unique::new(ptr),
                len: len,
                cap: len,
                inner: f(slice::from_raw_parts(ptr, len)),
            }
        };
        mem::forget(buf);
        res
    }

    /// Creates an OwningByteBuf from a boxed slice and a constructing function that may fail
    pub fn from_box_res<'a, F, E>(mut buf: Box<[u8]>,
                                  f: F)
                                  -> Result<OwningByteBuf<T>, (E, Box<[u8]>)>
        where F: FnOnce(&'a [u8]) -> Result<T, E>
    {
        let res = unsafe {
            let ptr = buf.as_mut_ptr();
            let len = buf.len();
            OwningByteBuf {
                resource: Unique::new(ptr),
                len: len,
                cap: len,
                inner: match f(slice::from_raw_parts(ptr, len)) {
                    Ok(t) => t,
                    Err(e) => return Err((e, buf)),
                },
            }
        };
        mem::forget(buf);
        Ok(res)
    }

    /// Returns a reference to the wrapped type.
    pub fn get(&self) -> &T {
        &self.inner
    }

    /// Drops the wrapped type and returns the underlying buffer back as a Vec.
    pub fn into_vec(mut self) -> Vec<u8> {
        let vec = {
            let OwningByteBuf { ref resource, len, cap, .. } = self;

            unsafe { Vec::from_raw_parts(**resource, len, cap) }
        };

        self.cap = 0;
        self.len = 0;

        vec
    }
}

impl<T> AsRef<T> for OwningByteBuf<T> {
    fn as_ref(&self) -> &T {
        self.get()
    }
}

impl<T> Drop for OwningByteBuf<T> {
    fn drop(&mut self) {
        let elem_size = mem::size_of::<u8>();
        let align = mem::align_of::<u8>();

        let num_bytes = elem_size * self.cap;
        if num_bytes > 0 {
            unsafe {
                heap::deallocate(*self.resource as *mut _, num_bytes, align);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Test<'a> {
        buf: &'a [u8],
    }

    #[test]
    fn test() {
        let foo = {
            let vec = vec![0, 1, 2, 3];
            OwningByteBuf::from_vec(vec, |buf| Test { buf: &buf[0..2] })
        };

        assert_eq!(foo.get().buf, &[0, 1]);

        let vec = foo.into_vec();

        assert_eq!(vec, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_res() {
        let vec = vec![0, 1, 2, 3];
        let res: Result<_, ((), _)> =
            OwningByteBuf::from_vec_res(vec, |buf| Ok(Test { buf: &buf[0..2] }));
        assert_eq!(res.unwrap().get().buf, &[0, 1]);

        let vec = vec![0, 1, 2, 3];
        let res: Result<OwningByteBuf<()>, _> = OwningByteBuf::from_vec_res(vec, |_| Err(()));
        assert!(res.is_err());
    }
}
