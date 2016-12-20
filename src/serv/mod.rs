use byteorder::{NetworkEndian, WriteBytesExt, ReadBytesExt};
use std::io::{Result, Write, Read};

pub trait Serv: Sized {
    fn send<W: Write>(&self, &mut W) -> Result<()>;
    fn receive<R: Read>(&mut R) -> Result<Self>;
}

impl Serv for String {
    fn send<W: Write>(&self, w: &mut W) -> Result<()> {
        self.len().send(w)?;
        w.write_all(self.as_bytes())
    }
    fn receive<R: Read>(r: &mut R) -> Result<Self> {
        let len = usize::receive(r)?;
        let mut v = Vec::with_capacity(len);
        v.resize(len, 0);
        r.read_exact(&mut v)?;
        Ok(String::from_utf8_lossy(&v).into_owned())
    }
}

impl<T: Serv> Serv for Vec<T> {
    fn send<W: Write>(&self, w: &mut W) -> Result<()> {
        self.len().send(w)?;
        for v in self{
            v.send(w)?;
        }
        Ok(())
    }
    fn receive<R: Read>(r: &mut R) -> Result<Self> {
        let len = usize::receive(r)?;
        let mut v = Vec::with_capacity(len);

        for _ in 0..len {
            v.push(T::receive(r)?);
        }

        Ok(v)
    }
}

impl Serv for u64 {
    fn send<W: Write>(&self, w: &mut W) -> Result<()> {
        w.write_u64::<NetworkEndian>(*self)
    }
    fn receive<R: Read>(r: &mut R) -> Result<Self> {
        r.read_u64::<NetworkEndian>()
    }
}

impl Serv for usize {
    fn send<W: Write>(&self, w: &mut W) -> Result<()> {
        (*self as u64).send(w)
    }
    fn receive<R: Read>(r: &mut R) -> Result<Self> {
        <u64>::receive(r).map(|v| v as usize)
    }
}

use std::mem;

impl Serv for f32 {
    fn send<W: Write>(&self, w: &mut W) -> Result<()> {
        let buf: [u8; 4] = unsafe {mem::transmute_copy(self)};
        w.write_all(&buf)
    }
    fn receive<R: Read>(r: &mut R) -> Result<Self> {
        let mut buf = [0; 4];
        r.read_exact(&mut buf)?;
        Ok(unsafe {mem::transmute(buf)})
    }
}

impl Serv for f64 {
    fn send<W: Write>(&self, w: &mut W) -> Result<()> {
        let buf: [u8; 8] = unsafe {mem::transmute_copy(self)};
        w.write_all(&buf)
    }
    fn receive<R: Read>(r: &mut R) -> Result<Self> {
        let mut buf = [0; 8];
        r.read_exact(&mut buf)?;
        Ok(unsafe {mem::transmute(buf)})
    }
}
