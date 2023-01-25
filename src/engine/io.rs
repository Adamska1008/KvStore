use crate::error::Result;
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};

pub struct BufReaderWithOffset<R: Read + Seek> {
    pub(crate) offset: u64,
    reader: BufReader<R>,
}

impl<R: Read + Seek> BufReaderWithOffset<R> {
    pub fn new(mut inner: R) -> Result<Self> {
        let offset = inner.seek(SeekFrom::Start(0))?;
        Ok(Self {
            offset,
            reader: BufReader::new(inner),
        })
    }
}

impl<R: Read + Seek> Seek for BufReaderWithOffset<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.offset = self.reader.seek(pos)?;
        Ok(self.offset)
    }
}

impl<R: Read + Seek> Read for BufReaderWithOffset<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.offset += len as u64;
        Ok(len)
    }
}

pub struct BufWriterWithOffset<W: Write + Seek> {
    pub(crate) offset: u64,
    pub(crate) writer: BufWriter<W>,
}

impl<W: Write + Seek> BufWriterWithOffset<W> {
    pub fn new(mut inner: W) -> Result<Self> {
        let offset = inner.seek(SeekFrom::Start(0))?;
        Ok(Self {
            offset,
            writer: BufWriter::new(inner),
        })
    }
}

impl<W: Write + Seek> Write for BufWriterWithOffset<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.offset += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
