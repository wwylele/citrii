pub use byte_struct_derive::*;

pub trait ByteStruct {
    fn write_bytes(&self, bytes: &mut [u8]);
    fn read_bytes(&mut self, bytes: &[u8]);
}

pub trait ByteStructImpl {
    fn byte_len() -> usize;
    fn write_le_bytes(&self, bytes: &mut [u8]);
    fn read_le_bytes(&mut self, bytes: &[u8]);
    fn write_be_bytes(&self, bytes: &mut [u8]);
    fn read_be_bytes(&mut self, bytes: &[u8]);
}

impl ByteStructImpl for u8 {
    fn byte_len() -> usize {1}
    fn write_le_bytes(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.clone().to_le_bytes()[..]);
    }
    fn read_le_bytes(&mut self, bytes: &[u8]) {
        *self = u8::from_le_bytes([bytes[0]]);
    }
    fn write_be_bytes(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.clone().to_be_bytes()[..]);
    }
    fn read_be_bytes(&mut self, bytes: &[u8]) {
        *self = u8::from_be_bytes([bytes[0]]);
    }
}

impl ByteStructImpl for i8 {
    fn byte_len() -> usize {1}
    fn write_le_bytes(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.clone().to_le_bytes()[..]);
    }
    fn read_le_bytes(&mut self, bytes: &[u8]) {
        *self = i8::from_le_bytes([bytes[0]]);
    }
    fn write_be_bytes(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.clone().to_be_bytes()[..]);
    }
    fn read_be_bytes(&mut self, bytes: &[u8]) {
        *self = i8::from_be_bytes([bytes[0]]);
    }
}

impl ByteStructImpl for u16 {
    fn byte_len() -> usize {2}
    fn write_le_bytes(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.clone().to_le_bytes()[..]);
    }
    fn read_le_bytes(&mut self, bytes: &[u8]) {
        *self = u16::from_le_bytes([bytes[0], bytes[1]]);
    }
    fn write_be_bytes(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.clone().to_be_bytes()[..]);
    }
    fn read_be_bytes(&mut self, bytes: &[u8]) {
        *self = u16::from_be_bytes([bytes[0], bytes[1]]);
    }
}

impl ByteStructImpl for i16 {
    fn byte_len() -> usize {2}
    fn write_le_bytes(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.clone().to_le_bytes()[..]);
    }
    fn read_le_bytes(&mut self, bytes: &[u8]) {
        *self = i16::from_le_bytes([bytes[0], bytes[1]]);
    }
    fn write_be_bytes(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.clone().to_be_bytes()[..]);
    }
    fn read_be_bytes(&mut self, bytes: &[u8]) {
        *self = i16::from_be_bytes([bytes[0], bytes[1]]);
    }
}

impl ByteStructImpl for u32 {
    fn byte_len() -> usize {4}
    fn write_le_bytes(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.clone().to_le_bytes()[..]);
    }
    fn read_le_bytes(&mut self, bytes: &[u8]) {
        *self = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    }
    fn write_be_bytes(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.clone().to_be_bytes()[..]);
    }
    fn read_be_bytes(&mut self, bytes: &[u8]) {
        *self = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    }
}

impl ByteStructImpl for i32 {
    fn byte_len() -> usize {4}
    fn write_le_bytes(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.clone().to_le_bytes()[..]);
    }
    fn read_le_bytes(&mut self, bytes: &[u8]) {
        *self = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    }
    fn write_be_bytes(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(&self.clone().to_be_bytes()[..]);
    }
    fn read_be_bytes(&mut self, bytes: &[u8]) {
        *self = i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    }
}

macro_rules! bsa0 {
    ($x:expr) => {
        impl<T: ByteStructImpl> ByteStructImpl for [T; $x] {
            fn byte_len() -> usize {($x) * T::byte_len()}
            fn write_le_bytes(&self, bytes: &mut [u8]) {
                let mut pos = 0;
                let len = T::byte_len();
                for i in 0 .. ($x) {
                    self[i].write_le_bytes(&mut bytes[pos .. pos + len]);
                    pos += len;
                }
            }
            fn read_le_bytes(&mut self, bytes: &[u8]) {
                let mut pos = 0;
                let len = T::byte_len();
                for i in 0 .. ($x) {
                    self[i].read_le_bytes(&bytes[pos .. pos + len]);
                    pos += len;
                }
            }
            fn write_be_bytes(&self, bytes: &mut [u8]) {
                let mut pos = 0;
                let len = T::byte_len();
                for i in 0 .. ($x) {
                    self[i].write_be_bytes(&mut bytes[pos .. pos + len]);
                    pos += len;
                }
            }
            fn read_be_bytes(&mut self, bytes: &[u8]) {
                let mut pos = 0;
                let len = T::byte_len();
                for i in 0 .. ($x) {
                    self[i].read_be_bytes(&bytes[pos .. pos + len]);
                    pos += len;
                }
            }
        }
    }
}

macro_rules! bsa1 { ($x:expr) => { bsa0!($x); bsa0!(1 + $x);}}
macro_rules! bsa2 { ($x:expr) => { bsa1!($x); bsa1!(2 + $x);}}
macro_rules! bsa3 { ($x:expr) => { bsa2!($x); bsa2!(4 + $x);}}
macro_rules! bsa4 { ($x:expr) => { bsa3!($x); bsa3!(8 + $x);}}
macro_rules! bsa5 { ($x:expr) => { bsa4!($x); bsa4!(16 + $x);}}
macro_rules! bsa6 { ($x:expr) => { bsa5!($x); bsa5!(32 + $x);}}
macro_rules! bsa7 { ($x:expr) => { bsa6!($x); bsa6!(64 + $x);}}
macro_rules! bsa8 { ($x:expr) => { bsa7!($x); bsa7!(128 + $x);}}

bsa8!(1);

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(ByteStructBE, PartialEq, Debug)]
    struct TestSubStruct {
        b: u16,
        c: u16,
    }

    #[derive(ByteStructLE, PartialEq, Debug)]
    struct TestStruct {
        a: u8,
        s: TestSubStruct,
        d: [u16; 3],
        e: u32,
    }

    #[test]
    fn it_works() {
        assert_eq!(TestStruct::byte_len(), 15);
        let mut data = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut s = TestStruct {
            a: 0x12,
            s: TestSubStruct {
                b: 0x3456,
                c: 0x78ff,
            },
            d: [0x1020, 0x3040, 0x5060],
            e: 0x9abcdef0,
        };
        s.write_bytes(&mut data[..]);
        assert_eq!(data, [0x12, 0x34, 0x56, 0x78, 0xff, 0x20, 0x10, 0x40, 0x30, 0x60, 0x50, 0xf0, 0xde, 0xbc, 0x9a]);

        data = [0x00, 0x11, 0x22, 0x33, 0x44, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd];
        s.read_bytes(&data[..]);
        assert_eq!(s, TestStruct {
            a: 0x00,
            s: TestSubStruct {
                b: 0x1122,
                c: 0x3344,
            },
            d: [0x5544, 0x7766, 0x9988],
            e: 0xddccbbaa,
        })

    }
}
