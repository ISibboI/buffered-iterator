use std::io::{ErrorKind, Read};

pub struct BufferedParser<Source> {
    buffer: Vec<u8>,
    source: Source,
}

impl<'a, Source: Read + 'a> Iterator for &'a mut BufferedParser<Source> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
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

        // Some(&self.buffer[0..length]) // does not compile, lifetime error
        unimplemented!("does not compile, lifetime error")
    }
}

impl<Source> BufferedParser<Source> {
    pub fn new(source: Source) -> Self {
        Self {
            buffer: Default::default(),
            source,
        }
    }
}
