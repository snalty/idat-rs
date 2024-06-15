use num_enum::TryFromPrimitive;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Clone, Copy)]
#[repr(usize)]
pub enum FieldType {
    SNPCount = 1000,
    IlluminaID = 102,
    SD = 103,
    Mean = 104,
    BeadCounts = 107,
    Midblock = 200,
    RunInfo = 300,
    RedGreen = 400,
    Manifest = 401,
    Barcode = 402,
    Format = 403,
    Label = 404,
    Opa = 405,
    SampleID = 406,
    Descr = 407,
    Plate = 408,
    Well = 409,
    Unknown = 410,
}

impl FieldType {
    pub fn get_data_type(&self) -> FieldFormat {
        match self {
            Self::SNPCount | Self::IlluminaID | Self::RedGreen => FieldFormat::Int,
            Self::SD | Self::BeadCounts | Self::Mean => FieldFormat::Short,
            Self::Manifest
            | Self::Barcode
            | Self::Format
            | Self::Unknown
            | Self::Descr
            | Self::Plate
            | Self::SampleID
            | Self::Opa
            | Self::Label
            | Self::Well => FieldFormat::String,
            Self::RunInfo => FieldFormat::RunInfo,
            Self::Midblock => FieldFormat::MidBlock,
        }
    }
}

#[derive(Debug)]
pub enum FieldFormat {
    String,
    Long,
    Short,
    Int,
    Byte,
    RunInfo,
    MidBlock,
}

#[derive(Debug)]
pub enum FieldValue {
    String(String),
    Long(i64),
    Short(u16),
    Int(i32),
    Byte(Vec<u8>),
    RunInfo,
    MidBlock,
}

#[derive(Debug, Clone, Copy)]
pub struct FieldDef {
    pub field_type: FieldType,
    pub byte_offset: u64,
}

#[derive(Debug)]
pub struct Field {
    pub field_type: FieldType,
    pub value: FieldValue,
}

#[cfg(test)]
mod tests {
    use std::{error::Error, path::Path};

    use super::*;

    #[test]
    fn test_determine_field_type() {
        let field_int = 1000;
        assert_eq!(FieldType::try_from(field_int).unwrap(), FieldType::SNPCount);
    }
}
