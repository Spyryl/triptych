#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    BigInt,
    Int16,
    Int32,
    Int64,
    Text,
    Bool,
    Json,
    TimestampTz,
    Date,
    Time,
    Uuid,
    Bytes,
    Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldSource {
    Persisted,
    Virtual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldDefault {
    None,
    Bool(bool),
    Int32(i32),
    Int64(i64),
    Text(&'static str),
    JsonObject,
    Now,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldDef {
    pub name: &'static str,
    pub field_type: FieldType,
    pub source: FieldSource,
    pub nullable: bool,
    pub required: bool,
    pub max_len: Option<usize>,
    pub default: FieldDefault,
    pub primary_key: bool,
}

impl FieldDef {
    pub const fn persisted(
        name: &'static str,
        field_type: FieldType,
        nullable: bool,
        required: bool,
        max_len: Option<usize>,
        default: FieldDefault,
        primary_key: bool,
    ) -> Self {
        Self {
            name,
            field_type,
            source: FieldSource::Persisted,
            nullable,
            required,
            max_len,
            default,
            primary_key,
        }
    }

    pub const fn virtual_field(
        name: &'static str,
        field_type: FieldType,
        nullable: bool,
        required: bool,
        max_len: Option<usize>,
        default: FieldDefault,
    ) -> Self {
        Self {
            name,
            field_type,
            source: FieldSource::Virtual,
            nullable,
            required,
            max_len,
            default,
            primary_key: false,
        }
    }

    pub const fn required_text(name: &'static str, max_len: usize) -> Self {
        Self::persisted(
            name,
            FieldType::Text,
            false,
            true,
            Some(max_len),
            FieldDefault::None,
            false,
        )
    }

    pub const fn optional_text(name: &'static str, max_len: Option<usize>) -> Self {
        Self::persisted(
            name,
            FieldType::Text,
            true,
            false,
            max_len,
            FieldDefault::None,
            false,
        )
    }

    pub const fn bigint_pk(name: &'static str) -> Self {
        Self::persisted(
            name,
            FieldType::BigInt,
            false,
            true,
            None,
            FieldDefault::Int64(0),
            true,
        )
    }
}
