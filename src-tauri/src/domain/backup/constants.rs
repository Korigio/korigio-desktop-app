pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const MANIFEST_NAME: &str = "manifest.json";
pub const DATABASE_ENTRY: &str = "database.sqlite";
pub const BACKUP_EXTENSION: &str = "backup";
/// Prefix for new backup filenames (`Korigio-YYYY-MM-DD-HHmm.backup`).
pub const BACKUP_FILE_PREFIX: &str = "Korigio";
/// Legacy prefix still accepted when checking whether a scheduled backup covers the period.
pub const BACKUP_FILE_PREFIX_LEGACY: &str = "Servioo";
