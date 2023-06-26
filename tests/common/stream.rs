use std::io::{Read, Write};

pub struct Stream {
    stream: Vec<u8>,
    pointer: usize,
}

impl Stream {
    pub fn new() -> Stream {
        Stream {
            stream: Vec::new(),
            pointer: 0,
        }
    }
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut i = 0;
        while i < buf.len() && self.pointer < self.stream.len() {
            buf[i] = self.stream[self.pointer];
            self.pointer += 1;
            i += 1;
        }
        Ok(i)
    }
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut i = 0;
        while i < buf.len() {
            self.stream.push(buf[i]);
            i += 1;
        }
        Ok(i)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
