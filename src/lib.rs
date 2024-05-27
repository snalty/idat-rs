use std::any::Any;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Iter, Path};

use fields::{FieldFormat, FieldType};

mod errors;
mod fields;

#[derive(Debug)]
pub struct Record {
    fields: Vec<fields::Field>
}

pub struct Reader {
    inner: BufReader<File>,
    fields: Vec<fields::FieldDef>,
}

impl Reader {
    pub fn new(mut inner: BufReader<File>) -> Result<Reader, errors::ReaderError> {
        // Check that this is actually an IDAT file
        Self::check_header(&mut inner)?;

        let mut version_buf = [0u8; 8];
        inner.read_exact(&mut version_buf)?;
        println!("{:?}", version_buf);
        let _version = u64::from_le_bytes(version_buf);

        let fields = Self::get_fields(&mut inner)?;

        println!("{:?}", fields);

        Ok(Reader { inner, fields })
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
        println!("{:?}", field_code);
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
                "IDAT" => println!("{}", s),
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
}

impl Iterator for Reader {
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.fill_buf().unwrap().is_empty() {
            return None
        };

        let mut fields: Vec<fields::Field> = Vec::with_capacity(self.fields.capacity());
        for field in &self.fields {
            match field.field_type.get_data_type() {
                FieldFormat::Int => {
                    let mut buf = [0u8; 4];
                    self.inner.read_exact(&mut buf).unwrap();

                    fields.push(
                        fields::Field {
                            field_type: field.field_type,
                            value: fields::FieldValue::Int(i32::from_le_bytes(buf))
                        }
                    )
                },
                FieldFormat::Long => {
                    let mut buf = [0u8; 8];
                    self.inner.read_exact(&mut buf).unwrap();

                    fields.push(
                        fields::Field {
                            field_type: field.field_type,
                            value: fields::FieldValue::Long(i64::from_le_bytes(buf))
                        }
                    )
                },
                FieldFormat::Short => {
                    let mut buf = [0u8; 2];
                    self.inner.read_exact(&mut buf).unwrap();

                    fields.push(
                        fields::Field {
                            field_type: field.field_type,
                            value: fields::FieldValue::Short(u16::from_le_bytes(buf))
                        }
                    )
                }
                _ => ()
            }
        };
        Some(
            Record {
            fields
        }
    )
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
        println!("{:?}", reader.next());
        Ok(())
    }
}
