use std::ops::{Deref, DerefMut};

use tokio_postgres::Transaction;

use crate::core::error::{CoreError, Result};
use crate::core::managed_record::{FieldValue, ManagedRecord};
use crate::core::managed_table::{FetchArgs, ManagedTable, hydrate_rows};

/// Transaction-only row-lock manager.
///
/// This mirrors the Node `ManagedTableLocking` boundary: regular table managers
/// should not issue `FOR UPDATE`. Classes that need row claims/serialized writes
/// can use this wrapper and must pass an active transaction per call.
pub struct ManagedTableLocking<R>
where
    R: ManagedRecord,
{
    manager: ManagedTable<R>,
}

impl<R> ManagedTableLocking<R>
where
    R: ManagedRecord,
{
    pub fn new(manager: ManagedTable<R>) -> Self {
        Self { manager }
    }

    pub fn into_inner(self) -> ManagedTable<R> {
        self.manager
    }

    pub async fn find_where_with_lock(
        &self,
        tx: &Transaction<'_>,
        args: FetchArgs<'_>,
    ) -> Result<Vec<R>> {
        if !args.has_where_clause() {
            return Err(CoreError::custom(
                "locked queries require an explicit where clause",
            ));
        }
        if args.has_view_key() {
            return Err(CoreError::custom(
                "locked queries must target the table, not a view key",
            ));
        }

        let sql = self.manager.build_table_lock_select_query(&args, true)?;
        let rows = self.manager.query_txn(tx, &sql, args.into_params()).await?;
        hydrate_rows(rows)
    }

    pub async fn find_one_with_lock(&self, tx: &Transaction<'_>, args: FetchArgs<'_>) -> Result<R> {
        let rows = self.find_where_with_lock(tx, args).await?;
        match rows.len() {
            1 => Ok(rows.into_iter().next().expect("one row exists")),
            0 => Err(CoreError::NotFound),
            count => Err(CoreError::custom(format!(
                "expected exactly one locked row, found {}",
                count
            ))),
        }
    }

    pub async fn find_one_with_lock_or_none(
        &self,
        tx: &Transaction<'_>,
        args: FetchArgs<'_>,
    ) -> Result<Option<R>> {
        let rows = self.find_where_with_lock(tx, args).await?;
        match rows.len() {
            0 => Ok(None),
            1 => Ok(rows.into_iter().next()),
            count => Err(CoreError::custom(format!(
                "expected at most one locked row, found {}",
                count
            ))),
        }
    }

    pub async fn acquire_advisory_xact_lock(
        tx: &Transaction<'_>,
        key: impl Into<String>,
    ) -> Result<()> {
        let key = key.into();
        if key.trim().is_empty() {
            return Err(CoreError::custom("advisory lock key is required"));
        }

        let params = vec![FieldValue::Text(key)];
        let param_refs: Vec<&(dyn postgres_types::ToSql + Sync)> = params
            .iter()
            .map(|v| v as &(dyn postgres_types::ToSql + Sync))
            .collect();
        tx.query(
            "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
            &param_refs,
        )
        .await?;
        Ok(())
    }
}

impl<R> Deref for ManagedTableLocking<R>
where
    R: ManagedRecord,
{
    type Target = ManagedTable<R>;

    fn deref(&self) -> &Self::Target {
        &self.manager
    }
}

impl<R> DerefMut for ManagedTableLocking<R>
where
    R: ManagedRecord,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.manager
    }
}
