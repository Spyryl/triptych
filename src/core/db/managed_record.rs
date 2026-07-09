use std::collections::BTreeMap;

use bytes::BytesMut;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use postgres_types::{IsNull, Json, ToSql, Type};
use rust_decimal::Decimal;
use serde_json::Value;
use tokio_postgres::Row;
use uuid::Uuid;

use crate::core::error::{CoreError, Result};

/// Typed field value used by records and managers.
///
/// This lives with the record contract because `*Rec` types use it to describe
/// their current field values. `ManagedTable` only consumes those values when it
/// needs to bind SQL parameters.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    Bool(bool),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    Decimal(Decimal),
    Text(String),
    Json(Value),
    Timestamp(DateTime<Utc>),
    NaiveTimestamp(NaiveDateTime),
    Date(NaiveDate),
    Time(NaiveTime),
    Uuid(Uuid),
    Bytes(Vec<u8>),
    Null,
}

impl FieldValue {
    pub fn null() -> Self {
        Self::Null
    }
}

impl ToSql for FieldValue {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> std::result::Result<IsNull, Box<dyn std::error::Error + Send + Sync>> {
        match self {
            Self::Bool(v) => v.to_sql(ty, out),
            Self::Int16(v) => v.to_sql(ty, out),
            Self::Int32(v) => v.to_sql(ty, out),
            Self::Int64(v) => v.to_sql(ty, out),
            Self::Float32(v) => v.to_sql(ty, out),
            Self::Float64(v) => v.to_sql(ty, out),
            Self::Decimal(v) => v.to_sql(ty, out),
            Self::Text(v) => v.to_sql(ty, out),
            Self::Json(v) => Json(v).to_sql(ty, out),
            Self::Timestamp(v) => v.to_sql(ty, out),
            Self::NaiveTimestamp(v) => v.to_sql(ty, out),
            Self::Date(v) => v.to_sql(ty, out),
            Self::Time(v) => v.to_sql(ty, out),
            Self::Uuid(v) => v.to_sql(ty, out),
            Self::Bytes(v) => v.to_sql(ty, out),
            Self::Null => Ok(IsNull::Yes),
        }
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }

    postgres_types::to_sql_checked!();
}

macro_rules! impl_field_value_from {
    ($variant:ident, $ty:ty) => {
        impl From<$ty> for FieldValue {
            fn from(value: $ty) -> Self {
                Self::$variant(value)
            }
        }
    };
}

impl_field_value_from!(Bool, bool);
impl_field_value_from!(Int16, i16);
impl_field_value_from!(Int32, i32);
impl_field_value_from!(Int64, i64);
impl_field_value_from!(Float32, f32);
impl_field_value_from!(Float64, f64);
impl_field_value_from!(Decimal, Decimal);
impl_field_value_from!(Json, Value);
impl_field_value_from!(Timestamp, DateTime<Utc>);
impl_field_value_from!(NaiveTimestamp, NaiveDateTime);
impl_field_value_from!(Date, NaiveDate);
impl_field_value_from!(Time, NaiveTime);
impl_field_value_from!(Uuid, Uuid);
impl_field_value_from!(Bytes, Vec<u8>);

impl From<&str> for FieldValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<String> for FieldValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl<T> From<Option<T>> for FieldValue
where
    T: Into<FieldValue>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(inner) => inner.into(),
            None => Self::Null,
        }
    }
}

/// Record contract required by ManagedTable.
///
/// Records own row hydration, persistence shape, validation, scrubbing, and dirty
/// state. The manager owns how those records move to and from PostgreSQL.
pub trait ManagedRecord: Clone + Send + Sync + 'static {
    fn from_row(row: &Row) -> Result<Self>;

    fn primary_key_value(&self) -> Option<FieldValue>;

    fn table_columns() -> &'static [&'static str];

    fn view_columns() -> &'static [&'static str] {
        Self::table_columns()
    }

    fn persistable_columns(&self) -> Vec<(&'static str, FieldValue)>;

    /// Returns only columns changed since the persisted snapshot.
    ///
    /// This is deliberately part of the core record contract: audit-safe updates
    /// must not write unchanged columns just because they exist on the record.
    fn modified_columns(&self) -> Vec<(&'static str, FieldValue)>;

    fn pk_field() -> &'static str {
        "id"
    }

    fn is_new(&self) -> bool {
        matches!(
            primary_key_state(self.primary_key_value()),
            Ok(PrimaryKeyState::New)
        )
    }

    fn is_valid(&self) -> Result<()> {
        Ok(())
    }

    fn scrub(&mut self) -> Result<()> {
        Ok(())
    }

    fn has_been_modified(&self) -> bool {
        !self.modified_columns().is_empty()
    }

    fn mark_clean(&mut self) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrimaryKeyState {
    New,
    Persisted,
}

pub(crate) fn primary_key_state(value: Option<FieldValue>) -> Result<PrimaryKeyState> {
    let Some(value) = value else {
        return Ok(PrimaryKeyState::New);
    };

    match value {
        FieldValue::Null => Ok(PrimaryKeyState::New),
        FieldValue::Int16(v) => numeric_primary_key_state(i64::from(v)),
        FieldValue::Int32(v) => numeric_primary_key_state(i64::from(v)),
        FieldValue::Int64(v) => numeric_primary_key_state(v),
        FieldValue::Text(v) if v.trim().is_empty() => Ok(PrimaryKeyState::New),
        FieldValue::Text(_) | FieldValue::Uuid(_) => Ok(PrimaryKeyState::Persisted),
        other => Err(CoreError::invalid_id(format!(
            "unsupported primary key value: {:?}",
            other
        ))),
    }
}

fn numeric_primary_key_state(value: i64) -> Result<PrimaryKeyState> {
    if value == 0 {
        Ok(PrimaryKeyState::New)
    } else if value > 0 {
        Ok(PrimaryKeyState::Persisted)
    } else {
        Err(CoreError::invalid_id(format!(
            "primary key cannot be negative: {}",
            value
        )))
    }
}

/// Shared dirty-tracking state for `*Rec` types.
///
/// Rust does not need a dynamic base record for field storage, but it does need
/// the same persisted-snapshot contract: updates must only write columns whose
/// values changed after the record was fetched or saved.
#[derive(Debug, Clone, Default)]
pub struct ManagedRecordState {
    original: BTreeMap<&'static str, FieldValue>,
    has_persisted_snapshot: bool,
    dirty: bool,
}

impl ManagedRecordState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn mark_clean(&mut self, current: Vec<(&'static str, FieldValue)>) {
        self.original = current.into_iter().collect();
        self.has_persisted_snapshot = true;
        self.dirty = false;
    }

    pub fn has_persisted_snapshot(&self) -> bool {
        self.has_persisted_snapshot
    }

    pub fn has_been_modified(&self, current: Vec<(&'static str, FieldValue)>) -> bool {
        if !self.has_persisted_snapshot {
            return true;
        }
        self.dirty
            || current
                .iter()
                .any(|(field, value)| self.original.get(field) != Some(value))
    }

    pub fn modified_columns(
        &self,
        current: Vec<(&'static str, FieldValue)>,
    ) -> Vec<(&'static str, FieldValue)> {
        if !self.has_persisted_snapshot {
            return current;
        }

        current
            .into_iter()
            .filter(|(field, value)| self.original.get(field) != Some(value))
            .collect()
    }
}
