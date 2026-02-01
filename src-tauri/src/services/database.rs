// Re-export from infrastructure layer for backward compatibility.
// Commands that are not yet migrated (family, member, category, export, statistics)
// still use this module.
pub use crate::infrastructure::persistence::database::{get_connection, get_db_path, init_database};
