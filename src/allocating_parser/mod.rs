use std::io::{ErrorKind, Read};

#[derive(Debug, Clone)]
pub struct AllocatingParser<Source> {
    source: Source,
}

impl<Source> AllocatingParser<Source> {
    pub fn new(source: Source) -> Self {
        Self { source }
    }
}

impl<Source: Read> Iterator for AllocatingParser<Source> {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        // Always allocate a new buffer
        let mut buffer = Vec::new();

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
        Some(buffer)
    }
}
