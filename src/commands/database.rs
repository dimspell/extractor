use super::Command;
use crate::cli::DatabaseCommands;
use crate::database::{
    bootstrap, create_all, import_all, import_databases, import_dialogues_paragraphs, import_maps,
    import_refs, import_rest, import_sprite_files,
};
use rusqlite::Connection;
use std::error::Error;
use std::path::Path;

/// Thin CLI wrapper around the `crate::database` module.
///
/// All schema knowledge, INSERT statements and import orchestration live in
/// the core library. This file only opens the connection, ensures the schema
/// exists and delegates to the appropriate import stage.
pub struct DatabaseCommand {
    pub subcommand: DatabaseCommands,
}

impl Command for DatabaseCommand {
    fn execute(&self) -> Result<(), Box<dyn Error>> {
        match &self.subcommand {
            DatabaseCommands::Import { game_path, db_path } => {
                let mut conn = Connection::open(db_path)?;
                bootstrap(&conn)?;
                eprintln!("Saving all data...");
                import_all(Path::new(game_path), &mut conn)?;
                conn.close().map_err(|(_conn, err)| err)?;
            }
            DatabaseCommands::DialogTexts { game_path, db_path } => {
                with_connection(db_path, |conn| {
                    import_dialogues_paragraphs(Path::new(game_path), conn)
                })?;
            }
            DatabaseCommands::Maps { game_path, db_path } => {
                with_connection(db_path, |conn| import_maps(Path::new(game_path), conn))?;
            }
            DatabaseCommands::Databases { game_path, db_path } => {
                with_connection(db_path, |conn| import_databases(Path::new(game_path), conn))?;
            }
            DatabaseCommands::Refs { game_path, db_path } => {
                with_connection(db_path, |conn| import_refs(Path::new(game_path), conn))?;
            }
            DatabaseCommands::Rest { game_path, db_path } => {
                with_connection(db_path, |conn| import_rest(Path::new(game_path), conn))?;
            }
            DatabaseCommands::Sprites { game_path, db_path } => {
                with_connection(db_path, |conn| {
                    import_sprite_files(Path::new(game_path), conn)
                })?;
            }
        }
        Ok(())
    }
}

/// Opens a connection, ensures the schema exists (idempotently, without
/// dropping any data), runs `f` and closes the connection.
fn with_connection(
    db_path: &str,
    f: impl FnOnce(&mut Connection) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let mut conn = Connection::open(db_path)?;
    create_all(&conn)?;
    f(&mut conn)?;
    conn.close().map_err(|(_conn, err)| err)?;
    Ok(())
}
