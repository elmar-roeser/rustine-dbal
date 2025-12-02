//! SQLite result set implementation

use crate::core::{Result, SqlValue};
use crate::driver::DriverResult;

/// SQLite query result
pub struct SqliteResult {
    rows: Vec<Vec<SqlValue>>,
    column_names: Vec<String>,
    rows_affected: u64,
    current_index: usize,
}

impl SqliteResult {
    /// Create a new result set
    pub(crate) fn new(rows: Vec<Vec<SqlValue>>, column_names: Vec<String>, rows_affected: u64) -> Self {
        Self {
            rows,
            column_names,
            rows_affected,
            current_index: 0,
        }
    }
}

impl DriverResult for SqliteResult {
    fn next_row(&mut self) -> Result<Option<Vec<SqlValue>>> {
        if self.current_index >= self.rows.len() {
            return Ok(None);
        }

        let row = self.rows[self.current_index].clone();
        self.current_index += 1;
        Ok(Some(row))
    }

    fn column_count(&self) -> usize {
        self.column_names.len()
    }

    fn column_names(&self) -> &[String] {
        &self.column_names
    }

    fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_iteration() {
        let rows = vec![
            vec![SqlValue::I64(1), SqlValue::String("Alice".to_string())],
            vec![SqlValue::I64(2), SqlValue::String("Bob".to_string())],
        ];
        let columns = vec!["id".to_string(), "name".to_string()];

        let mut result = SqliteResult::new(rows, columns, 0);

        // First row
        let row1 = result.next_row().unwrap();
        assert!(row1.is_some());
        let row1 = row1.unwrap();
        assert_eq!(row1[0], SqlValue::I64(1));

        // Second row
        let row2 = result.next_row().unwrap();
        assert!(row2.is_some());
        let row2 = row2.unwrap();
        assert_eq!(row2[0], SqlValue::I64(2));

        // No more rows
        let row3 = result.next_row().unwrap();
        assert!(row3.is_none());
    }

    #[test]
    fn test_column_info() {
        let columns = vec!["id".to_string(), "name".to_string(), "age".to_string()];
        let result = SqliteResult::new(Vec::new(), columns, 5);

        assert_eq!(result.column_count(), 3);
        assert_eq!(result.column_names(), &["id", "name", "age"]);
        assert_eq!(result.rows_affected(), 5);
    }

    #[test]
    fn test_all_rows() {
        let rows = vec![
            vec![SqlValue::I64(1)],
            vec![SqlValue::I64(2)],
            vec![SqlValue::I64(3)],
        ];

        let mut result = SqliteResult::new(rows, vec!["num".to_string()], 0);
        let all = result.all_rows().unwrap();

        assert_eq!(all.len(), 3);
        assert_eq!(all[0][0], SqlValue::I64(1));
        assert_eq!(all[1][0], SqlValue::I64(2));
        assert_eq!(all[2][0], SqlValue::I64(3));
    }
}
