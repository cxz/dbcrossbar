//! Table-related support for BigQuery.

use serde_json;
use std::io::Write;

use super::{BqColumn, ColumnBigQueryExt, Ident, TableName, Usage};
use crate::common::*;
use crate::schema::{Column, Table};

/// Extensions to `Column` (the portable version) to handle BigQuery-query
/// specific stuff.
pub(crate) trait TableBigQueryExt {
    /// Can we import data into this table directly from a CSV file?
    fn bigquery_can_import_from_csv(&self) -> Result<bool>;
}

impl TableBigQueryExt for Table {
    fn bigquery_can_import_from_csv(&self) -> Result<bool> {
        for col in &self.columns {
            if !col.bigquery_can_import_from_csv()? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// A BigQuery table schema.
pub(crate) struct BqTable {
    /// The BigQuery name of this table.
    name: TableName,
    /// The columns of this table.
    columns: Vec<BqColumn>,
}

impl BqTable {
    /// Give a BigQuery `TableName`, a database-independent list of `Columns`,
    /// and the intended usage within BigQuery, map them to a corresponding
    /// `BqTable`.
    ///
    /// We require the BigQuery `TableName` to be passed in separately, because
    /// using the table name from the database-independent `Table` has tended to
    /// be a source of bugs in the past.
    pub(crate) fn for_table_name_and_columns(
        name: TableName,
        columns: &[Column],
        usage: Usage,
    ) -> Result<BqTable> {
        let columns = columns
            .iter()
            .map(|c| BqColumn::for_column(c, usage))
            .collect::<Result<Vec<BqColumn>>>()?;
        Ok(BqTable { name, columns })
    }

    /// Get the BigQuery table name for this table.
    pub(crate) fn name(&self) -> &TableName {
        &self.name
    }

    /// Write out this table as a JSON schema.
    pub(crate) fn write_json_schema(&self, f: &mut dyn Write) -> Result<()> {
        serde_json::to_writer_pretty(f, &self.columns)?;
        Ok(())
    }

    /// Generate SQL which `SELECT`s from a temp table, and fixes the types
    /// of columns that couldn't be imported from CSVs.
    ///
    /// This `BqTable` should have been created with `Usage::FinalTable`.
    pub(crate) fn write_import_sql(&self, f: &mut dyn Write) -> Result<()> {
        for (i, col) in self.columns.iter().enumerate() {
            col.write_import_udf(f, i)?;
        }
        write!(f, "SELECT ")?;
        for (i, col) in self.columns.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            col.write_import_select_expr(f, i)?;
        }
        write!(f, " FROM {}", Ident(&self.name.dotted().to_string()))?;
        Ok(())
    }
}
