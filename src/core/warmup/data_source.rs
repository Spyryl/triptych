use std::collections::BTreeMap;

use crate::core::error::{CoreError, Result};
use crate::core::managed_table::ManagedTableConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataObjectType {
    Table,
    View,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSourceRole {
    Table,
    DefaultView,
    View,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSourceDef {
    pub record_key: String,
    pub schema_name: String,
    pub object_name: String,
    pub object_type: DataObjectType,
    pub role: DataSourceRole,
    pub alias_key: Option<String>,
    pub active: bool,
}

impl DataSourceDef {
    pub fn source_name(&self) -> String {
        format!("{}.{}", self.schema_name, self.object_name)
    }

    pub fn table(
        record_key: impl Into<String>,
        schema_name: impl Into<String>,
        object_name: impl Into<String>,
    ) -> Self {
        Self {
            record_key: record_key.into(),
            schema_name: schema_name.into(),
            object_name: object_name.into(),
            object_type: DataObjectType::Table,
            role: DataSourceRole::Table,
            alias_key: None,
            active: true,
        }
    }

    pub fn default_view(
        record_key: impl Into<String>,
        schema_name: impl Into<String>,
        object_name: impl Into<String>,
    ) -> Self {
        Self {
            record_key: record_key.into(),
            schema_name: schema_name.into(),
            object_name: object_name.into(),
            object_type: DataObjectType::View,
            role: DataSourceRole::DefaultView,
            alias_key: None,
            active: true,
        }
    }

    pub fn view(
        record_key: impl Into<String>,
        alias_key: impl Into<String>,
        schema_name: impl Into<String>,
        object_name: impl Into<String>,
    ) -> Self {
        Self {
            record_key: record_key.into(),
            schema_name: schema_name.into(),
            object_name: object_name.into(),
            object_type: DataObjectType::View,
            role: DataSourceRole::View,
            alias_key: Some(alias_key.into()),
            active: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RecordDataSources {
    pub table: Option<DataSourceDef>,
    pub default_view: Option<DataSourceDef>,
    views_by_alias: BTreeMap<String, DataSourceDef>,
    views_by_source: BTreeMap<String, DataSourceDef>,
}

impl RecordDataSources {
    pub fn table_source(&self) -> Result<&DataSourceDef> {
        self.table
            .as_ref()
            .ok_or_else(|| CoreError::custom("record data source has no table configured"))
    }

    pub fn default_view_source(&self) -> Result<&DataSourceDef> {
        self.default_view
            .as_ref()
            .ok_or_else(|| CoreError::custom("record data source has no default view configured"))
    }

    pub fn view_sources(&self) -> impl Iterator<Item = &DataSourceDef> {
        self.views_by_source.values()
    }

    pub fn view_alias_pairs(&self) -> impl Iterator<Item = (&str, &DataSourceDef)> {
        self.views_by_alias
            .iter()
            .map(|(alias, source)| (alias.as_str(), source))
    }

    pub fn insert(&mut self, source: DataSourceDef) {
        if !source.active {
            return;
        }

        match source.role {
            DataSourceRole::Table => {
                self.table = Some(source);
            }
            DataSourceRole::DefaultView => {
                self.views_by_source
                    .insert(source.source_name(), source.clone());
                self.default_view = Some(source);
            }
            DataSourceRole::View => {
                if let Some(alias_key) = &source.alias_key {
                    self.views_by_alias
                        .insert(alias_key.clone(), source.clone());
                }
                self.views_by_source.insert(source.source_name(), source);
            }
        }
    }

    pub fn resolve_view(&self, requested: Option<&str>) -> Result<&DataSourceDef> {
        let Some(requested) = requested else {
            return self.default_view_source();
        };

        if let Some(source) = self.views_by_alias.get(requested) {
            return Ok(source);
        }

        if let Some(source) = self.views_by_source.get(requested) {
            return Ok(source);
        }

        Err(CoreError::custom(format!(
            "view is not allowed for record data source: {}",
            requested
        )))
    }
}

#[derive(Debug, Clone, Default)]
pub struct DataSourceRegistry {
    records: BTreeMap<String, RecordDataSources>,
}

impl DataSourceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_sources(sources: impl IntoIterator<Item = DataSourceDef>) -> Self {
        let mut registry = Self::new();
        for source in sources {
            registry.insert(source);
        }
        registry
    }

    pub fn insert(&mut self, source: DataSourceDef) {
        self.records
            .entry(source.record_key.clone())
            .or_default()
            .insert(source);
    }

    pub fn record(&self, record_key: &str) -> Result<&RecordDataSources> {
        self.records.get(record_key).ok_or_else(|| {
            CoreError::custom(format!(
                "record data source is not configured: {}",
                record_key
            ))
        })
    }
}

impl ManagedTableConfig {
    pub fn from_record_sources(sources: &RecordDataSources) -> Result<Self> {
        let table_name = sources.table_source()?.source_name();
        let default_view_name = sources.default_view_source()?.source_name();
        let mut config = Self::new(table_name, default_view_name);

        for source in sources.view_sources() {
            config = config.allow_view(source.source_name());
        }

        for (alias, source) in sources.view_alias_pairs() {
            config = config.with_view_alias(alias, source.source_name());
        }

        Ok(config)
    }
}
