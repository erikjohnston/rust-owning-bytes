#![feature(unique, alloc, heap_api, question_mark)]

extern crate alloc;

use std::mem;
use std::ptr::Unique;
use std::slice;
use alloc::heap;

use std::convert::AsRef;


pub struct OwningByteBuf<T> {
    resource: Unique<u8>,
    len: usize,
    cap: usize,
    inner: T,
}

impl<T> OwningByteBuf<T> {
    pub fn from_vec<'a, F>(mut buf: Vec<u8>, f: F) -> OwningByteBuf<T> where F: FnOnce(&'a[u8]) -> T {
        let res = unsafe {
            let ptr = buf.as_mut_ptr();
            let len = buf.len();
            let cap = buf.capacity();
            OwningByteBuf {
                resource: Unique::new(ptr),
                len: len,
                cap: cap,
                inner: f(slice::from_raw_parts(ptr, len)),
            }
        };
        mem::forget(buf);
        res
    }

    pub fn from_vec_res<'a, F, E>(mut buf: Vec<u8>, f: F) -> Result<OwningByteBuf<T>, (E, Vec<u8>)>
        where F: FnOnce(&'a[u8]) -> Result<T, E>
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

    pub fn from_box<'a, F>(mut buf: Box<[u8]>, f: F) -> OwningByteBuf<T> where F: FnOnce(&'a[u8]) -> T {
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

    pub fn from_box_res<'a, F, E>(mut buf: Box<[u8]>, f: F) -> Result<OwningByteBuf<T>, (E, Box<[u8]>)>
        where F: FnOnce(&'a[u8]) -> Result<T, E>
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

    pub fn get(&self) -> &T {
        &self.inner
    }

    pub fn into_vec(mut self) -> Vec<u8> {
        let vec = {
            let OwningByteBuf { ref resource, len, cap, .. } = self;

            unsafe {
                Vec::from_raw_parts(**resource, len, cap)
            }
        };

        self.cap = 0;
        self.len = 0;

        vec
    }
}

impl <T> AsRef<T> for OwningByteBuf<T> {
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
        buf: &'a [u8]
    }

    #[test]
    fn test() {
        let foo = {
            let vec = vec![0,1,2,3];
            OwningByteBuf::from_vec(vec, |buf| Test { buf: &buf[0..2] } )
        };

        assert_eq!(foo.get().buf, &[0, 1]);

        let vec = foo.into_vec();

        assert_eq!(vec, vec![0,1,2,3]);
    }

    #[test]
    fn test_res() {
        let vec = vec![0,1,2,3];
        let res: Result<_,((),_)> = OwningByteBuf::from_vec_res(vec, |buf| Ok(Test { buf: &buf[0..2] }) );
        assert_eq!(res.unwrap().get().buf, &[0, 1]);

        let vec = vec![0,1,2,3];
        let res: Result<OwningByteBuf<()>, _> = OwningByteBuf::from_vec_res(vec, |_| Err(()) );
        assert!(res.is_err());
    }
}
