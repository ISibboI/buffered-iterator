use core::fmt::{Debug, Formatter};
use core::ops::Deref;
use std::io::{ErrorKind, Read};
use std::rc::Rc;

#[derive(Clone)]
pub struct Entry {
    buffer: Rc<Vec<u8>>,
    length: usize,
}

impl Deref for Entry {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buffer[..self.length]
    }
}

impl Debug for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}

#[derive(Debug)]
pub struct BufferedParser<Source> {
    buffer: Rc<Vec<u8>>,
    source: Source,
}

impl<Source> BufferedParser<Source> {
    pub fn new(source: Source) -> Self {
        Self {
            buffer: Default::default(),
            source,
        }
    }
}

impl<Source: Clone> Clone for BufferedParser<Source> {
    fn clone(&self) -> Self {
        BufferedParser::new(self.source.clone())
    }
}

impl<Source: Read> Iterator for BufferedParser<Source> {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        // If there are still undropped entries, we need a new buffer.
        // Otherwise, we would update the buffer under the entries feet, making them invalid.
        if Rc::strong_count(&self.buffer) > 1 {
            self.buffer = Default::default();
        }
        let buffer = Rc::get_mut(&mut self.buffer).unwrap();

        // Parsing code...
        if buffer.is_empty() {
            buffer.resize(1, 0);
        }

        match &self.source.read_exact(&mut buffer[0..1]) {
            Ok(_) => {}
            err @ Err(error) => {
                if error.kind() == ErrorKind::UnexpectedEof {
                    return None;
                } else {
                    err.as_ref().unwrap();
                }
            }
        };
        let length = u8::from_be_bytes(buffer[0..1].try_into().unwrap()) as usize;

        if buffer.len() < length {
            buffer.resize(length, 0);
        }
        self.source.read_exact(&mut buffer[0..length]).unwrap();

        // Give the entry a pointer to the buffer, and the length of the current entry.
        Some(Entry {
            buffer: self.buffer.clone(),
            length,
        })
    }
}
