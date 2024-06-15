use std::any::Any;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Iter, Path};

use fields::{Field, FieldDef, FieldFormat, FieldType, FieldValue};

mod errors;
mod fields;

#[derive(Debug)]
pub struct Record {
    pub data: Vec<fields::Field>,
    pub fields: Vec<fields::FieldDef>,
}

pub struct Reader {
    inner: BufReader<File>,
    fields: Vec<fields::FieldDef>,
    snp_count: u32
}

impl Reader {
    pub fn new(mut inner: BufReader<File>) -> Result<Reader, errors::ReaderError> {
        // Check that this is actually an IDAT file
        Self::check_header(&mut inner)?;

        let mut version_buf = [0u8; 8];
        inner.read_exact(&mut version_buf)?;

        let _version = u64::from_le_bytes(version_buf);

        let fields = Self::get_fields(&mut inner)?;

        let mut snp_count_buf = [0u8; 4];
        inner.seek(SeekFrom::Start(
            fields
                .iter()
                .find(|f| f.field_type == FieldType::SNPCount)
                .unwrap()
                .byte_offset,
        ))?;
        inner.read_exact(&mut snp_count_buf);

        let snp_count = u32::from_le_bytes(snp_count_buf);

        Ok(Reader { inner, fields, snp_count })
    }

    fn get_fields(
        inner: &mut BufReader<File>,
    ) -> Result<Vec<fields::FieldDef>, errors::ReaderError> {
        let mut fields_buf = [0u8; 4];
        inner.read_exact(&mut fields_buf)?;
        let field_count = u32::from_le_bytes(fields_buf);

        (0..field_count)
            .map(|_| Self::get_field_definition(inner))
            .collect()
    }

    fn get_field_definition(
        inner: &mut BufReader<File>,
    ) -> Result<fields::FieldDef, errors::ReaderError> {
        // Read the field code, it is a short (i.e one byte)
        let mut field_code_buf = [0u8; 2];
        inner.read_exact(&mut field_code_buf)?;
        let field_code = u16::from_le_bytes(field_code_buf);

        let field_type = match fields::FieldType::try_from(field_code as usize) {
            Ok(f) => f,
            Err(_) => fields::FieldType::Unknown,
        };

        // Read the offset
        let mut offset_buf = [0u8; 8];
        inner.read_exact(&mut offset_buf)?;
        let byte_offset = u64::from_le_bytes(offset_buf);

        Ok(fields::FieldDef {
            field_type,
            byte_offset,
        })
    }

    fn check_header(inner: &mut BufReader<File>) -> Result<(), errors::ReaderError> {
        let mut string_header: [u8; 4] = [0; 4];
        inner.read_exact(&mut string_header)?;

        match String::from_utf8(string_header.to_vec()) {
            Ok(s) => match s.as_str() {
                "IDAT" => (),
                _ => return Err(errors::ReaderError::InvalidHeader { actual: s }),
            },
            Err(_) => {
                return Err(errors::ReaderError::InvalidHeader {
                    actual: format!("{:?}", string_header),
                })
            }
        };

        Ok(())
    }

    fn field_iter(&mut self, field: fields::FieldType) -> Result<FieldIterator, errors::ReaderError> {
        match field {
            FieldType::IlluminaID | FieldType::SD | FieldType::Mean | FieldType::BeadCounts => (),
            _ => return Err(errors::ReaderError::FieldNotIterable)
        }

        let field_def = match self.fields.iter().find(|f| f.field_type == field) {
            Some(&field) => field,
            None => return Err(errors::ReaderError::MissingField { field })
        };

        return Ok(FieldIterator::new(self, field_def))
    }
}

struct FieldIterator<'a> {
    reader: &'a mut Reader,
    field_def: FieldDef,
    returned: usize,
    offset: u64,
}

impl <'a> FieldIterator<'_> {
    pub fn new(mut reader: &'a mut Reader, field_def: FieldDef) -> FieldIterator<'a> {
        FieldIterator{ 
            reader,
            field_def,
            returned: 0,
            offset: field_def.byte_offset
        }
    }
}


impl Iterator for FieldIterator<'_> {
    type Item = FieldValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.returned > self.reader.snp_count as usize {
            return None
        };

        self.returned += 1;
         
        // Make sure we are at the correct place in the file.
        match self.reader.inner.stream_position() {
            Ok(pos) => {
                if pos != self.offset {
                    self.reader.inner.seek(SeekFrom::Start(self.offset)).unwrap();
                }
            },
            Err(e) => return None
        }

        let value = match self.field_def.field_type.get_data_type() {
            FieldFormat::Int => {
                let mut buf = [0u8; 4];
                self.reader.inner.read_exact(&mut buf).unwrap();
                Some(FieldValue::Int(i32::from_le_bytes(buf)))
            },
            _ => todo!("Other fields not yet implemented")
        };

        self.offset = self.reader.inner.stream_position().expect("valid reader");
        self.returned += 1;

        value
    }
}

pub struct Builder;

impl Builder {
    pub fn from_path(src: &Path) -> Result<Reader, errors::ReaderError> {
        let r = BufReader::new(File::open(src)?);
        Self::build_from_reader(r)
    }

    pub fn build_from_reader(reader: BufReader<File>) -> Result<Reader, errors::ReaderError> {
        Reader::new(reader)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_determine_file_type() -> Result<(), errors::ReaderError> {
        let path = Path::new("/Users/samnalty/Developer/idat-rs/200144450018_R04C01_Red.idat");
        let mut reader = Builder::from_path(path)?;
        let record: Vec<FieldValue> = reader.field_iter(FieldType::IlluminaID)?.collect();
        println!("{:?}", record);
        Ok(())
    }
}
