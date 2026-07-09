use std::borrow::Cow;
use std::collections::BTreeMap;

use deadpool_postgres::Pool;
use postgres_types::ToSql;
use tokio_postgres::{Row, Transaction};

use crate::core::error::{CoreError, Result};
use crate::core::managed_record::{FieldValue, ManagedRecord, PrimaryKeyState, primary_key_state};

fn to_param_refs(params: &[FieldValue]) -> Vec<&(dyn ToSql + Sync)> {
    params.iter().map(|v| v as &(dyn ToSql + Sync)).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderDirection {
    Asc,
    Desc,
}

impl OrderDirection {
    fn as_sql(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrderSpec {
    pub field: String,
    pub direction: OrderDirection,
}

impl OrderSpec {
    pub fn asc(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            direction: OrderDirection::Asc,
        }
    }

    pub fn desc(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            direction: OrderDirection::Desc,
        }
    }
}

/// Constructor configuration for a table class.
#[derive(Debug, Clone)]
pub struct ManagedTableConfig {
    pub table_name: String,
    pub pk_field_name: String,
    pub view_name: String,
    pub alias_file_name: String,
    pub allowed_view_names: Vec<String>,
    pub view_aliases: BTreeMap<String, String>,
}

impl ManagedTableConfig {
    pub fn new(table_name: impl Into<String>, view_name: impl Into<String>) -> Self {
        let view_name = view_name.into();
        Self {
            table_name: table_name.into(),
            pk_field_name: "id".to_string(),
            view_name: view_name.clone(),
            alias_file_name: "me".to_string(),
            allowed_view_names: vec![view_name],
            view_aliases: BTreeMap::new(),
        }
    }

    pub fn with_pk_field_name(mut self, pk_field_name: impl Into<String>) -> Self {
        self.pk_field_name = pk_field_name.into();
        self
    }

    pub fn with_alias_file_name(mut self, alias_file_name: impl Into<String>) -> Self {
        self.alias_file_name = alias_file_name.into();
        self
    }

    pub fn allow_view(mut self, view_name: impl Into<String>) -> Self {
        self.allowed_view_names.push(view_name.into());
        self
    }

    pub fn with_view_alias(
        mut self,
        alias: impl Into<String>,
        view_name: impl Into<String>,
    ) -> Self {
        let view_name = view_name.into();
        self.allowed_view_names.push(view_name.clone());
        self.view_aliases.insert(alias.into(), view_name);
        self
    }
}

/// Rust equivalent of Axiom ManagedTable / WinDev wxManagerClass.
///
/// Table classes should configure this in their constructor and avoid duplicating
/// fetch/save logic. Reads are view-first. Writes always target the table.
pub struct ManagedTable<R>
where
    R: ManagedRecord,
{
    pool: Pool,
    config: ManagedTableConfig,
    records: Vec<R>,
}

impl<R> ManagedTable<R>
where
    R: ManagedRecord,
{
    pub fn new(pool: Pool, config: ManagedTableConfig) -> Result<Self> {
        validate_sql_source_name(&config.table_name, "table_name")?;
        validate_sql_source_name(&config.view_name, "view_name")?;
        validate_sql_identifier(&config.pk_field_name, "pk_field_name")?;
        validate_sql_identifier(&config.alias_file_name, "alias_file_name")?;
        if config.pk_field_name != R::pk_field() {
            return Err(CoreError::custom(format!(
                "table pk_field_name does not match record pk_field for {}",
                config.table_name
            )));
        }

        for view_name in &config.allowed_view_names {
            validate_sql_source_name(view_name, "allowed_view_names")?;
        }

        for alias in config.view_aliases.keys() {
            validate_sql_identifier(alias, "view_aliases.alias")?;
        }

        Ok(Self {
            pool,
            config,
            records: Vec::new(),
        })
    }

    pub fn table_name(&self) -> &str {
        &self.config.table_name
    }

    pub fn view_name(&self) -> &str {
        &self.config.view_name
    }

    pub fn pk_field_name(&self) -> &str {
        &self.config.pk_field_name
    }

    pub fn alias_file_name(&self) -> &str {
        &self.config.alias_file_name
    }

    pub fn records(&self) -> &[R] {
        &self.records
    }

    pub fn records_mut(&mut self) -> &mut [R] {
        &mut self.records
    }

    pub fn replace_records(&mut self, records: Vec<R>) {
        self.records = records;
    }

    pub async fn fetch_view(&mut self, args: FetchArgs<'_>) -> Result<Vec<R>> {
        let source = self.resolve_view_key(args.view_key)?;
        let rows = self.fetch_from_source(&source, args).await?;
        self.records = hydrate_rows(rows)?;
        Ok(self.records.clone())
    }

    pub async fn fetch_table(&mut self, args: FetchArgs<'_>) -> Result<Vec<R>> {
        let rows = self
            .fetch_from_source(&self.config.table_name, args)
            .await?;
        self.records = hydrate_rows(rows)?;
        Ok(self.records.clone())
    }

    pub async fn build_view(&mut self, args: FetchArgs<'_>) -> Result<Vec<R>> {
        self.fetch_view(args).await
    }

    pub async fn build_table(&mut self, args: FetchArgs<'_>) -> Result<Vec<R>> {
        self.fetch_table(args).await
    }

    pub fn build_view_select_query(&self, args: &FetchArgs<'_>) -> Result<String> {
        let source = self.resolve_view_key(args.view_key)?;
        self.build_select_query(&source, R::view_columns(), args)
    }

    pub fn build_table_select_query(&self, args: &FetchArgs<'_>) -> Result<String> {
        self.build_select_query(&self.config.table_name, R::table_columns(), args)
    }

    pub async fn find_by_id_from_view(&self, id: impl Into<FieldValue>) -> Result<Option<R>> {
        self.find_by_id_from_source(self.view_name(), R::view_columns(), id.into())
            .await
    }

    pub async fn find_by_id_from_table(&self, id: impl Into<FieldValue>) -> Result<Option<R>> {
        self.find_by_id_from_source(self.table_name(), R::table_columns(), id.into())
            .await
    }

    pub async fn insert_rec(&self, rec: R) -> Result<R> {
        let mut rec = rec;
        rec.scrub()?;
        rec.is_valid()?;

        let include_pk = matches!(
            primary_key_state(rec.primary_key_value())?,
            PrimaryKeyState::Persisted
        );
        let (columns, values) = self.persistable_columns(&rec, include_pk)?;

        if columns.is_empty() {
            return Err(CoreError::custom("no columns to insert"));
        }

        let placeholders: Vec<String> = (1..=values.len()).map(|i| format!("${}", i)).collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
            self.config.table_name,
            columns.join(", "),
            placeholders.join(", ")
        );

        let rows = self.query(&sql, values).await?;
        let mut inserted: R = hydrate_single_row(rows, "insert")?;
        inserted.mark_clean();
        Ok(inserted)
    }

    pub async fn update_rec(&self, rec: R) -> Result<R> {
        let mut rec = rec;
        rec.scrub()?;
        rec.is_valid()?;

        let id = rec
            .primary_key_value()
            .ok_or_else(|| CoreError::invalid_id("record must have a primary key for update"))?;

        if primary_key_state(Some(id.clone()))? != PrimaryKeyState::Persisted {
            return Err(CoreError::invalid_id(
                "record must have a persisted positive primary key for update",
            ));
        }

        let (columns, mut values) = self.modified_columns(&rec)?;

        if columns.is_empty() {
            rec.mark_clean();
            return Ok(rec);
        }

        let set_fragments: Vec<String> = columns
            .iter()
            .enumerate()
            .map(|(idx, column)| format!("{} = ${}", column, idx + 1))
            .collect();

        values.push(id);

        let sql = format!(
            "UPDATE {} SET {} WHERE {} = ${} RETURNING *",
            self.config.table_name,
            set_fragments.join(", "),
            self.config.pk_field_name,
            values.len()
        );

        let rows = self.query(&sql, values).await?;
        let mut updated: R = hydrate_single_row(rows, "update")?;
        updated.mark_clean();
        Ok(updated)
    }

    pub async fn save_rec(&self, rec: R) -> Result<R> {
        match primary_key_state(rec.primary_key_value())? {
            PrimaryKeyState::New => self.insert_rec(rec).await,
            PrimaryKeyState::Persisted => self.update_rec(rec).await,
        }
    }

    pub async fn save_txn_rec(&self, tx: &Transaction<'_>, rec: R) -> Result<R> {
        match primary_key_state(rec.primary_key_value())? {
            PrimaryKeyState::New => self.insert_txn_rec(tx, rec).await,
            PrimaryKeyState::Persisted => self.update_txn_rec(tx, rec).await,
        }
    }

    async fn insert_txn_rec(&self, tx: &Transaction<'_>, mut rec: R) -> Result<R> {
        rec.scrub()?;
        rec.is_valid()?;

        let include_pk = matches!(
            primary_key_state(rec.primary_key_value())?,
            PrimaryKeyState::Persisted
        );
        let (columns, values) = self.persistable_columns(&rec, include_pk)?;

        if columns.is_empty() {
            return Err(CoreError::custom("no columns to insert"));
        }

        let placeholders: Vec<String> = (1..=values.len()).map(|i| format!("${}", i)).collect();
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
            self.config.table_name,
            columns.join(", "),
            placeholders.join(", ")
        );

        let rows = self.query_txn(tx, &sql, values).await?;
        let mut inserted: R = hydrate_single_row(rows, "insert")?;
        inserted.mark_clean();
        Ok(inserted)
    }

    async fn update_txn_rec(&self, tx: &Transaction<'_>, mut rec: R) -> Result<R> {
        rec.scrub()?;
        rec.is_valid()?;

        let id = rec
            .primary_key_value()
            .ok_or_else(|| CoreError::invalid_id("record must have a primary key for update"))?;

        if primary_key_state(Some(id.clone()))? != PrimaryKeyState::Persisted {
            return Err(CoreError::invalid_id(
                "record must have a persisted positive primary key for update",
            ));
        }

        let (columns, mut values) = self.modified_columns(&rec)?;

        if columns.is_empty() {
            rec.mark_clean();
            return Ok(rec);
        }

        let set_fragments: Vec<String> = columns
            .iter()
            .enumerate()
            .map(|(idx, column)| format!("{} = ${}", column, idx + 1))
            .collect();

        values.push(id);

        let sql = format!(
            "UPDATE {} SET {} WHERE {} = ${} RETURNING *",
            self.config.table_name,
            set_fragments.join(", "),
            self.config.pk_field_name,
            values.len()
        );

        let rows = self.query_txn(tx, &sql, values).await?;
        let mut updated: R = hydrate_single_row(rows, "update")?;
        updated.mark_clean();
        Ok(updated)
    }

    pub async fn save_modified_records(&self, records: &mut [R]) -> Result<Vec<R>> {
        let mut saved = Vec::new();

        for rec in records.iter_mut() {
            if rec.has_been_modified() {
                let persisted = self.save_rec(rec.clone()).await?;
                *rec = persisted.clone();
                saved.push(persisted);
            }
        }

        Ok(saved)
    }

    pub async fn delete_by_id(&self, id: impl Into<FieldValue>) -> Result<bool> {
        let sql = format!(
            "DELETE FROM {} WHERE {} = $1",
            self.config.table_name, self.config.pk_field_name
        );
        let affected = self.execute(&sql, vec![id.into()]).await?;
        Ok(affected > 0)
    }

    pub(crate) fn resolve_view_key(&self, requested: Option<&str>) -> Result<String> {
        let requested = match requested {
            Some(value) => value,
            None => return Ok(self.config.view_name.clone()),
        };

        if let Some(view_name) = self.config.view_aliases.get(requested) {
            return Ok(view_name.clone());
        }

        if self
            .config
            .allowed_view_names
            .iter()
            .any(|view_name| view_name == requested)
        {
            return Ok(requested.to_string());
        }

        Err(CoreError::custom(format!(
            "view key is not allowed for {}: {}",
            self.config.table_name, requested
        )))
    }

    async fn find_by_id_from_source(
        &self,
        source: &str,
        columns: &'static [&'static str],
        id: FieldValue,
    ) -> Result<Option<R>> {
        let select = qualify_columns(columns, self.alias_file_name())?;
        let sql = format!(
            "SELECT {} FROM {} AS {} WHERE {}.{} = $1 LIMIT 1",
            select,
            source,
            self.config.alias_file_name,
            self.config.alias_file_name,
            self.config.pk_field_name
        );
        let rows = self.query(&sql, vec![id]).await?;
        rows.first()
            .map(|row| {
                let mut rec = R::from_row(row)?;
                rec.mark_clean();
                Ok(rec)
            })
            .transpose()
    }

    async fn fetch_from_source(&self, source: &str, args: FetchArgs<'_>) -> Result<Vec<Row>> {
        let default_columns = if source == self.config.table_name {
            R::table_columns()
        } else {
            R::view_columns()
        };
        let sql = self.build_select_query(source, default_columns, &args)?;
        self.query(&sql, args.params).await
    }

    fn build_select_query(
        &self,
        source: &str,
        default_columns: &'static [&'static str],
        args: &FetchArgs<'_>,
    ) -> Result<String> {
        validate_sql_source_name(source, "source")?;

        let alias_name = args.alias_name.unwrap_or(&self.config.alias_file_name);
        validate_sql_identifier(alias_name, "alias_name")?;

        let default_select = qualify_columns(default_columns, alias_name)?;
        let select = args.select.unwrap_or(default_select.as_str());
        validate_select_clause(select)?;

        if let Some(join_clause) = args.join.as_deref() {
            validate_trusted_clause(join_clause, "join")?;
        }

        if let Some(where_clause) = args.where_clause.as_deref() {
            validate_trusted_clause(where_clause, "where_clause")?;
        }

        let mut sql = format!("SELECT {} FROM {} AS {}", select, source, alias_name);

        if let Some(join_clause) = args.join.as_deref() {
            sql.push(' ');
            sql.push_str(join_clause);
        }

        if let Some(where_clause) = args.where_clause.as_deref() {
            sql.push_str(" WHERE ");
            sql.push_str(where_clause);
        }

        if !args.order_by.is_empty() {
            let mut parts = Vec::with_capacity(args.order_by.len());
            for order in &args.order_by {
                validate_sql_field_ref(&order.field, "order_by.field")?;
                parts.push(format!("{} {}", order.field, order.direction.as_sql()));
            }
            sql.push_str(" ORDER BY ");
            sql.push_str(&parts.join(", "));
        }

        if let Some(limit) = args.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = args.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        Ok(sql)
    }

    fn persistable_columns(
        &self,
        rec: &R,
        include_pk: bool,
    ) -> Result<(Vec<String>, Vec<FieldValue>)> {
        let mut columns = Vec::new();
        let mut values = Vec::new();

        for (field, value) in rec.persistable_columns() {
            if !include_pk && field == self.config.pk_field_name {
                continue;
            }
            self.validate_write_column(field)?;
            columns.push(field.to_string());
            values.push(value);
        }

        Ok((columns, values))
    }

    fn modified_columns(&self, rec: &R) -> Result<(Vec<String>, Vec<FieldValue>)> {
        let mut columns = Vec::new();
        let mut values = Vec::new();

        let persistable_columns = rec.persistable_columns();
        let raw_modified_columns = rec.modified_columns();
        if raw_modified_columns.len() == persistable_columns.len()
            && raw_modified_columns
                .iter()
                .any(|(field, _)| *field == self.config.pk_field_name)
        {
            return Err(CoreError::custom(format!(
                "persisted update for {} requires a persisted record snapshot",
                self.config.table_name
            )));
        }

        for (field, value) in raw_modified_columns {
            if field == self.config.pk_field_name {
                continue;
            }
            self.validate_write_column(field)?;
            columns.push(field.to_string());
            values.push(value);
        }

        Ok((columns, values))
    }

    fn validate_write_column(&self, field: &str) -> Result<()> {
        validate_sql_identifier(field, "write_column")?;
        if R::table_columns().contains(&field) {
            return Ok(());
        }

        Err(CoreError::custom(format!(
            "write column is not declared for {}: {}",
            self.config.table_name, field
        )))
    }

    async fn query(&self, sql: &str, params: Vec<FieldValue>) -> Result<Vec<Row>> {
        let client = self.pool.get().await?;
        let param_refs = to_param_refs(&params);
        Ok(client.query(sql, &param_refs[..]).await?)
    }

    pub(crate) async fn query_txn(
        &self,
        tx: &Transaction<'_>,
        sql: &str,
        params: Vec<FieldValue>,
    ) -> Result<Vec<Row>> {
        let param_refs = to_param_refs(&params);
        Ok(tx.query(sql, &param_refs[..]).await?)
    }

    async fn execute(&self, sql: &str, params: Vec<FieldValue>) -> Result<u64> {
        let client = self.pool.get().await?;
        let param_refs = to_param_refs(&params);
        Ok(client.execute(sql, &param_refs[..]).await?)
    }

    pub(crate) fn build_table_lock_select_query(
        &self,
        args: &FetchArgs<'_>,
        for_update: bool,
    ) -> Result<String> {
        let mut sql = self.build_select_query(&self.config.table_name, R::table_columns(), args)?;
        if for_update {
            sql.push_str(" FOR UPDATE");
        }
        Ok(sql)
    }
}

#[derive(Debug, Clone)]
pub struct FetchArgs<'a> {
    view_key: Option<&'a str>,
    alias_name: Option<&'a str>,
    select: Option<&'a str>,
    join: Option<Cow<'a, str>>,
    where_clause: Option<Cow<'a, str>>,
    params: Vec<FieldValue>,
    order_by: Vec<OrderSpec>,
    limit: Option<i64>,
    offset: Option<i64>,
}

impl<'a> Default for FetchArgs<'a> {
    fn default() -> Self {
        Self {
            view_key: None,
            alias_name: None,
            select: None,
            join: None,
            where_clause: None,
            params: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }
}

impl<'a> FetchArgs<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn where_eq(field: &'a str, value: impl Into<FieldValue>) -> Result<Self> {
        validate_sql_field_ref(field, "where_eq.field")?;
        Ok(Self {
            where_clause: Some(Cow::Owned(format!("{} = $1", field))),
            params: vec![value.into()],
            ..Self::default()
        })
    }

    pub fn view_key(mut self, view_key: &'a str) -> Self {
        self.view_key = Some(view_key);
        self
    }

    pub fn view_name(mut self, view_name: &'a str) -> Self {
        self.view_key = Some(view_name);
        self
    }

    pub fn alias_name(mut self, alias_name: &'a str) -> Self {
        self.alias_name = Some(alias_name);
        self
    }

    pub(crate) fn where_clause(mut self, where_clause: impl Into<Cow<'a, str>>) -> Self {
        self.where_clause = Some(where_clause.into());
        self
    }

    pub(crate) fn has_where_clause(&self) -> bool {
        self.where_clause.is_some()
    }

    pub(crate) fn has_view_key(&self) -> bool {
        self.view_key.is_some()
    }

    pub(crate) fn into_params(self) -> Vec<FieldValue> {
        self.params
    }

    pub fn with_param(mut self, value: impl Into<FieldValue>) -> Self {
        self.params.push(value.into());
        self
    }

    pub fn order_by(mut self, order: OrderSpec) -> Self {
        self.order_by.push(order);
        self
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }
}

pub(crate) fn hydrate_rows<R>(rows: Vec<Row>) -> Result<Vec<R>>
where
    R: ManagedRecord,
{
    let mut records = Vec::with_capacity(rows.len());
    for row in rows {
        let mut rec = R::from_row(&row)?;
        rec.scrub()?;
        rec.mark_clean();
        records.push(rec);
    }
    Ok(records)
}

fn hydrate_single_row<R>(rows: Vec<Row>, operation: &str) -> Result<R>
where
    R: ManagedRecord,
{
    let row = rows
        .first()
        .ok_or_else(|| CoreError::custom(format!("{} did not return a row", operation)))?;
    let mut rec = R::from_row(row)?;
    rec.scrub()?;
    Ok(rec)
}

fn validate_sql_identifier(value: &str, label: &str) -> Result<()> {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(CoreError::custom(format!("{} cannot be blank", label)));
    };

    if !(first == '_' || first.is_ascii_alphabetic()) {
        return Err(CoreError::custom(format!(
            "{} is not a safe SQL identifier: {}",
            label, value
        )));
    }

    if chars.any(|ch| !(ch == '_' || ch.is_ascii_alphanumeric())) {
        return Err(CoreError::custom(format!(
            "{} is not a safe SQL identifier: {}",
            label, value
        )));
    }

    Ok(())
}

fn validate_sql_source_name(value: &str, label: &str) -> Result<()> {
    let parts: Vec<&str> = value.split('.').collect();
    if parts.is_empty() || parts.len() > 2 {
        return Err(CoreError::custom(format!(
            "{} is not a safe SQL source name: {}",
            label, value
        )));
    }

    for part in parts {
        validate_sql_identifier(part, label)?;
    }

    Ok(())
}

fn validate_select_clause(value: &str) -> Result<()> {
    if value == "*" {
        return Ok(());
    }

    for part in value.split(',') {
        validate_select_part(part.trim())?;
    }

    Ok(())
}

fn qualify_columns(columns: &'static [&'static str], alias_name: &str) -> Result<String> {
    validate_sql_identifier(alias_name, "alias_name")?;
    let mut qualified = Vec::with_capacity(columns.len());
    for column in columns {
        validate_sql_identifier(column, "column")?;
        qualified.push(format!("{}.{}", alias_name, column));
    }
    Ok(qualified.join(", "))
}

fn validate_select_part(value: &str) -> Result<()> {
    let lower = value.to_ascii_lowercase();
    let parts: Vec<&str> = lower.split(" as ").collect();
    match parts.as_slice() {
        [field] => validate_sql_field_ref(field.trim(), "select"),
        [field, alias] => {
            validate_sql_field_ref(field.trim(), "select.field")?;
            validate_sql_identifier(alias.trim(), "select.alias")
        }
        _ => Err(CoreError::custom(format!(
            "select contains unsafe field expression: {}",
            value
        ))),
    }
}

fn validate_sql_field_ref(value: &str, label: &str) -> Result<()> {
    let parts: Vec<&str> = value.split('.').collect();
    if parts.is_empty() || parts.len() > 2 {
        return Err(CoreError::custom(format!(
            "{} is not a safe SQL field reference: {}",
            label, value
        )));
    }

    for part in parts {
        validate_sql_identifier(part, label)?;
    }

    Ok(())
}

fn validate_trusted_clause(value: &str, label: &str) -> Result<()> {
    if value.contains(';') || value.contains("--") || value.contains("/*") || value.contains("*/") {
        return Err(CoreError::custom(format!(
            "{} contains unsafe SQL text",
            label
        )));
    }

    Ok(())
}
