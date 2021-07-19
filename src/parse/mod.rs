use crate::parse::model::{u1, u2, u4, ClassFile, CpInfo};

mod model;

struct ParseErr;

struct Data {
    data: Vec<u1>,
    pointer: usize,
}

impl From<Vec<u8>> for Data {
    fn from(data: Vec<u1>) -> Self {
        Self { data, pointer: 0 }
    }
}

impl Data {
    fn u1(&mut self) -> Result<u1, ParseErr> {
        let item = self.data.get(self.pointer).cloned();
        self.pointer += 1;
        item.ok_or(ParseErr)
    }

    fn u2(&mut self) -> Result<u2, ParseErr> {
        Ok(((self.u1()? as u2) << 8) | self.u1() as u2)
    }

    fn u4(&mut self) -> Result<u4, ParseErr> {
        Ok(((self.u2()? as u4) << 16) | self.u2() as u4)
    }

    fn last_u1(&mut self) -> Result<u1, ParseErr> {
        self.data.get(self.pointer - 1).cloned().ok_or(ParseErr)
    }

    fn last_u2(&mut self) -> Result<u2, ParseErr> {
        let last2u1 = self.data.get(self.pointer - 2).cloned().ok_or(ParseErr);
        Ok(((self.last_u1()? as u2) << 8) | last2u1 as u2)
    }
}

pub trait Parse {
    fn parse(data: &mut Data) -> Result<Self, ParseErr>;
}

pub trait ParseVec<T> {
    fn parse_vec<T: Parse>(data: &mut Data, len: usize) -> Result<Self, ParseErr>;
}

impl Parse for ClassFile {
    fn parse(data: &mut Data) -> Result<Self, ParseErr> {
        Ok(Self {
            magic: data.u4()?,
            minor_version: data.u2()?,
            major_version: data.u2()?,
            constant_pool_count: data.u2()?,
            constant_pool: Vec::parse_vec::<CpInfo>(data, data.last_u2() as usize)?,
            access_flags: data.u2()?,
            this_class: data.u2()?,
            super_class: data.u2()?,
            interfaces_count: data.u2()?,
            interfaces: vec![],
            fields_count: 0,
            fields: vec![],
            method_count: 0,
            methods: vec![],
            attributes_count: 0,
            attributes: vec![],
        })
    }
}

impl<T: Parse> ParseVec<T> for Vec<T> {
    fn parse_vec<T: Parse>(data: &mut Data, len: usize) -> Result<Self, ParseErr> {
        let mut vec = Vec::with_capacity(len);
        for i in 0..len {
            vec.push(T::parse(data)?);
        }
        Ok(vec)
    }
}
