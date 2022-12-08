use std::fmt::{Debug, Formatter};
use std::io::{ErrorKind, Read};
use std::ops::Deref;
use std::rc::Rc;
use std::{mem, slice};

pub struct BufferedParser<Source> {
    buffer: Vec<u8>,
    source: Source,
    borrow_counter: Rc<()>,
}

#[derive(Clone)]
pub struct Entry {
    offset: *const u8,
    len: usize,

    // I hope the compiler does not optimise this away...
    #[allow(dead_code)]
    borrow_counter: Rc<()>,
}

impl<Source: Read> Iterator for BufferedParser<Source> {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        // Making sure that all instances of Entry have been dropped.
        let rc = mem::replace(&mut self.borrow_counter, Rc::new(()));
        Rc::try_unwrap(rc).unwrap();

        #[allow(clippy::len_zero)]
        if self.buffer.len() < 1 {
            self.buffer.resize(1, 0);
        }
        match &self.source.read_exact(&mut self.buffer[0..1]) {
            Ok(_) => {}
            err @ Err(error) => {
                if error.kind() == ErrorKind::UnexpectedEof {
                    return None;
                } else {
                    err.as_ref().unwrap();
                }
            }
        };
        let length = u8::from_be_bytes(self.buffer[0..1].try_into().unwrap()) as usize;

        if self.buffer.len() < length {
            self.buffer.resize(length, 0);
        }
        self.source.read_exact(&mut self.buffer[0..length]).unwrap();

        let slice = &self.buffer[0..length];
        let range = slice.as_ptr_range();
        Some(Entry {
            offset: range.start,
            len: slice.len(),
            borrow_counter: self.borrow_counter.clone(),
        })
    }
}

impl<Source> Drop for BufferedParser<Source> {
    fn drop(&mut self) {
        // Making sure that all instances of Entry have been dropped.
        let rc = mem::replace(&mut self.borrow_counter, Rc::new(()));
        Rc::try_unwrap(rc).unwrap();
    }
}

impl Deref for Entry {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.offset, self.len) }
    }
}

impl Debug for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let slice: &[u8] = self.deref();
        slice.fmt(f)
    }
}

impl<Source> BufferedParser<Source> {
    pub fn new(source: Source) -> Self {
        Self {
            buffer: Default::default(),
            source,
            borrow_counter: Default::default(),
        }
    }
}
