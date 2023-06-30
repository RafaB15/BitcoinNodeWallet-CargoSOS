use std::io::{Read, Write};

pub struct Stream {
    write_stream: Vec<u8>,
    read_stream: Vec<u8>,
    pointer: usize,
}

impl Stream {
    pub fn new(read_stream: Vec<u8>) -> Stream {
        Stream {
            read_stream,
            write_stream: Vec::new(),
            pointer: 0,
        }
    }

    pub fn get_write_stream(&self) -> Stream {
        Stream {
            read_stream: self.write_stream.clone(),
            write_stream: Vec::new(),
            pointer: 0,
        }
    }
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut i = 0;
        while i < buf.len() && self.pointer < self.read_stream.len() {
            buf[i] = self.read_stream[self.pointer];
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
            self.write_stream.push(buf[i]);
            i += 1;
        }
        Ok(i)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
