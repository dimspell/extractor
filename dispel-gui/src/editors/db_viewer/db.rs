use rusqlite::Connection;
use std::collections::HashMap;

// ─── Column metadata from PRAGMA table_info ─────────────────────────────────
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_pk: bool,
}

// ─── Sort direction ─────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDir {
    Asc,
    Desc,
}

impl SortDir {
    pub fn toggle(self) -> Self {
        match self {
            SortDir::Asc => SortDir::Desc,
            SortDir::Desc => SortDir::Asc,
        }
    }

    pub fn sql(&self) -> &str {
        match self {
            SortDir::Asc => "ASC",
            SortDir::Desc => "DESC",
        }
    }

    pub fn arrow(&self) -> &str {
        match self {
            SortDir::Asc => " ▲",
            SortDir::Desc => " ▼",
        }
    }
}

// ─── The result of a query ──────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Vec<String>>,
    pub total_rows: usize,
}

// ─── Cell edit tracking ─────────────────────────────────────────────────────
/// (row_index, col_index) -> new value
pub type PendingEdits = HashMap<(usize, usize), String>;

// ─── Public API (all blocking – called from async Tasks) ────────────────────

/// List all user tables in the database.
pub fn list_tables(db_path: &str) -> Result<Vec<String>, String> {
    let conn = open(db_path)?;
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .map_err(|e| e.to_string())?;

    let names: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(names)
}

/// Fetch column metadata for a table.
pub fn table_columns(db_path: &str, table: &str) -> Result<Vec<ColumnInfo>, String> {
    let conn = open(db_path)?;
    let sql = format!("PRAGMA table_info(\"{}\")", escape_ident(table));
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let cols: Vec<ColumnInfo> = stmt
        .query_map([], |row| {
            Ok(ColumnInfo {
                name: row.get::<_, String>(1)?,
                data_type: row.get::<_, String>(2).unwrap_or_default(),
                is_pk: row.get::<_, i32>(5).unwrap_or(0) > 0,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(cols)
}

/// Execute a SELECT query, returning column info + rows as strings.
pub fn execute_query(
    db_path: &str,
    sql: &str,
    limit: usize,
    offset: usize,
) -> Result<QueryResult, String> {
    let conn = open(db_path)?;

    // Count total rows for the base query (wrap user sql in a subquery)
    let count_sql = format!("SELECT COUNT(*) FROM ({sql})");
    let total_rows: usize = conn
        .query_row(&count_sql, [], |r| r.get::<_, i64>(0))
        .unwrap_or(0) as usize;

    // The actual paginated fetch
    let paged_sql = format!("{sql} LIMIT {limit} OFFSET {offset}");
    let mut stmt = conn.prepare(&paged_sql).map_err(|e| e.to_string())?;

    let col_count = stmt.column_count();
    let col_names: Vec<String> = (0..col_count)
        .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
        .collect();

    // Build column info (we don't know PK from arbitrary queries)
    let columns: Vec<ColumnInfo> = col_names
        .iter()
        .map(|n| ColumnInfo {
            name: n.clone(),
            data_type: String::new(),
            is_pk: false,
        })
        .collect();

    let rows: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            let mut cells = Vec::with_capacity(col_count);
            for i in 0..col_count {
                let val: String = match row.get_ref(i) {
                    Ok(rusqlite::types::ValueRef::Null) => "NULL".into(),
                    Ok(rusqlite::types::ValueRef::Integer(v)) => v.to_string(),
                    Ok(rusqlite::types::ValueRef::Real(v)) => v.to_string(),
                    Ok(rusqlite::types::ValueRef::Text(v)) => {
                        String::from_utf8_lossy(v).to_string()
                    }
                    Ok(rusqlite::types::ValueRef::Blob(v)) => {
                        format!("[blob {} bytes]", v.len())
                    }
                    Err(_) => "?".into(),
                };
                cells.push(val);
            }
            Ok(cells)
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(QueryResult {
        columns,
        rows,
        total_rows,
    })
}

/// Build a `SELECT * FROM table` with optional search filter and sorting.
pub fn build_table_query(
    table: &str,
    columns: &[ColumnInfo],
    search: &str,
    sort_col: Option<usize>,
    sort_dir: SortDir,
) -> String {
    let table_escaped = escape_ident(table);
    let mut sql = format!("SELECT * FROM \"{}\"", table_escaped);

    if !search.is_empty() {
        let conditions: Vec<String> = columns
            .iter()
            .map(|c| {
                format!(
                    "CAST(\"{}\" AS TEXT) LIKE '%{}%'",
                    escape_ident(&c.name),
                    escape_value(search)
                )
            })
            .collect();
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" OR "));
    }

    if let Some(idx) = sort_col {
        if idx < columns.len() {
            sql.push_str(&format!(
                " ORDER BY \"{}\" {}",
                escape_ident(&columns[idx].name),
                sort_dir.sql()
            ));
        }
    }

    sql
}

/// Commit pending edits to the database.
pub fn commit_edits(
    db_path: &str,
    table: &str,
    columns: &[ColumnInfo],
    original_rows: &[Vec<String>],
    edits: &PendingEdits,
) -> Result<usize, String> {
    let conn = open(db_path)?;

    // Find primary key columns
    let pk_cols: Vec<(usize, &ColumnInfo)> = columns
        .iter()
        .enumerate()
        .filter(|(_, c)| c.is_pk)
        .collect();

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let mut count = 0;

    // Group edits by row index
    let mut row_edits: HashMap<usize, Vec<(usize, &String)>> = HashMap::new();
    for ((row_idx, col_idx), value) in edits {
        row_edits
            .entry(*row_idx)
            .or_default()
            .push((*col_idx, value));
    }

    for (row_idx, cell_edits) in &row_edits {
        let set_clause: Vec<String> = cell_edits
            .iter()
            .map(|(ci, val)| {
                format!(
                    "\"{}\" = '{}'",
                    escape_ident(&columns[*ci].name),
                    escape_value(val)
                )
            })
            .collect();

        let where_clause = if !pk_cols.is_empty() {
            pk_cols
                .iter()
                .map(|(pi, pc)| {
                    format!(
                        "\"{}\" = '{}'",
                        escape_ident(&pc.name),
                        escape_value(&original_rows[*row_idx][*pi])
                    )
                })
                .collect::<Vec<_>>()
                .join(" AND ")
        } else {
            // Fallback: match on ALL original columns (risky but functional)
            columns
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let orig = &original_rows[*row_idx][i];
                    if orig == "NULL" {
                        format!("\"{}\" IS NULL", escape_ident(&c.name))
                    } else {
                        format!("\"{}\" = '{}'", escape_ident(&c.name), escape_value(orig))
                    }
                })
                .collect::<Vec<_>>()
                .join(" AND ")
        };

        let sql = format!(
            "UPDATE \"{}\" SET {} WHERE {} LIMIT 1",
            escape_ident(table),
            set_clause.join(", "),
            where_clause
        );

        tx.execute(&sql, [])
            .map_err(|e| format!("Row {}: {}", row_idx, e))?;
        count += 1;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(count)
}

/// Write the current result set to a CSV file.
pub fn export_csv(path: &str, columns: &[ColumnInfo], rows: &[Vec<String>]) -> Result<(), String> {
    let mut wtr = csv::Writer::from_path(path).map_err(|e| e.to_string())?;

    // Header
    let headers: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();
    wtr.write_record(&headers).map_err(|e| e.to_string())?;

    // Data
    for row in rows {
        wtr.write_record(row).map_err(|e| e.to_string())?;
    }

    wtr.flush().map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Internals ──────────────────────────────────────────────────────────────

fn open(db_path: &str) -> Result<Connection, String> {
    Connection::open(db_path).map_err(|e| format!("Cannot open database '{}': {}", db_path, e))
}

fn escape_ident(s: &str) -> String {
    s.replace('"', "\"\"")
}

fn escape_value(s: &str) -> String {
    s.replace('\'', "''")
}
