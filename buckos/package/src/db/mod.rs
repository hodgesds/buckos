//! Package database for tracking installed packages
//!
//! Uses SQLite for reliable, ACID-compliant storage of package metadata.

pub mod collision;

pub use collision::*;

use crate::{Error, InstalledFile, InstalledPackage, PackageId, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashSet;
use std::path::Path;

/// Package database
pub struct PackageDb {
    conn: Connection,
}

impl PackageDb {
    /// Open or create the package database
    pub fn open(path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db_file = path.join("packages.db");
        let conn = Connection::open(&db_file)?;

        let db = Self { conn };
        db.init_schema()?;

        Ok(db)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            -- Installed packages table
            CREATE TABLE IF NOT EXISTS packages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category TEXT NOT NULL,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                slot TEXT NOT NULL DEFAULT '0',
                installed_at TEXT NOT NULL,
                size INTEGER NOT NULL DEFAULT 0,
                build_time INTEGER NOT NULL DEFAULT 0,
                explicit INTEGER NOT NULL DEFAULT 1,
                UNIQUE(category, name, slot)
            );

            -- Package USE flags
            CREATE TABLE IF NOT EXISTS package_use_flags (
                package_id INTEGER NOT NULL,
                flag TEXT NOT NULL,
                FOREIGN KEY (package_id) REFERENCES packages(id) ON DELETE CASCADE,
                PRIMARY KEY (package_id, flag)
            );

            -- Installed files
            CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                package_id INTEGER NOT NULL,
                path TEXT NOT NULL UNIQUE,
                file_type INTEGER NOT NULL,
                mode INTEGER NOT NULL,
                size INTEGER NOT NULL,
                blake3_hash TEXT,
                mtime INTEGER NOT NULL,
                FOREIGN KEY (package_id) REFERENCES packages(id) ON DELETE CASCADE
            );

            -- Dependencies
            CREATE TABLE IF NOT EXISTS dependencies (
                package_id INTEGER NOT NULL,
                dep_category TEXT NOT NULL,
                dep_name TEXT NOT NULL,
                dep_slot TEXT,
                build_time INTEGER NOT NULL DEFAULT 0,
                run_time INTEGER NOT NULL DEFAULT 1,
                FOREIGN KEY (package_id) REFERENCES packages(id) ON DELETE CASCADE,
                PRIMARY KEY (package_id, dep_category, dep_name)
            );

            -- Indices
            CREATE INDEX IF NOT EXISTS idx_packages_name ON packages(name);
            CREATE INDEX IF NOT EXISTS idx_packages_category ON packages(category);
            CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
            CREATE INDEX IF NOT EXISTS idx_deps_dep ON dependencies(dep_category, dep_name);

            -- Triggers for referential integrity
            PRAGMA foreign_keys = ON;
            "#,
        )?;

        Ok(())
    }

    /// Check if a package is installed
    pub fn is_installed(&self, name: &str) -> Result<bool> {
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM packages WHERE name = ?",
            params![name],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Get an installed package by name
    pub fn get_installed(&self, name: &str) -> Result<Option<InstalledPackage>> {
        let pkg = self
            .conn
            .query_row(
                "SELECT id, category, name, version, slot, installed_at, size, build_time, explicit
                 FROM packages WHERE name = ?",
                params![name],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, u64>(6)?,
                        row.get::<_, bool>(7)?,
                        row.get::<_, bool>(8)?,
                    ))
                },
            )
            .optional()?;

        match pkg {
            Some((id, category, name, version, slot, installed_at, size, build_time, explicit)) => {
                let version =
                    semver::Version::parse(&version).map_err(|_| Error::InvalidVersion(version))?;
                let installed_at = chrono::DateTime::parse_from_rfc3339(&installed_at)
                    .map_err(|e| Error::DatabaseError(e.to_string()))?
                    .with_timezone(&chrono::Utc);

                let use_flags = self.get_package_use_flags(id)?;
                let files = self.get_package_files_by_id(id)?;

                Ok(Some(InstalledPackage {
                    id: PackageId::new(category, name.clone()),
                    name,
                    version,
                    slot,
                    installed_at,
                    use_flags,
                    files,
                    size,
                    build_time,
                    explicit,
                }))
            }
            None => Ok(None),
        }
    }

    /// Get all installed packages
    pub fn get_all_installed(&self) -> Result<Vec<InstalledPackage>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, category, name, version, slot, installed_at, size, build_time, explicit
             FROM packages ORDER BY category, name",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, u64>(6)?,
                row.get::<_, bool>(7)?,
                row.get::<_, bool>(8)?,
            ))
        })?;

        let mut packages = Vec::new();
        for row in rows {
            let (id, category, name, version, slot, installed_at, size, build_time, explicit) =
                row?;
            let version =
                semver::Version::parse(&version).map_err(|_| Error::InvalidVersion(version))?;
            let installed_at = chrono::DateTime::parse_from_rfc3339(&installed_at)
                .map_err(|e| Error::DatabaseError(e.to_string()))?
                .with_timezone(&chrono::Utc);

            let use_flags = self.get_package_use_flags(id)?;
            let files = self.get_package_files_by_id(id)?;

            packages.push(InstalledPackage {
                id: PackageId::new(category, name.clone()),
                name,
                version,
                slot,
                installed_at,
                use_flags,
                files,
                size,
                build_time,
                explicit,
            });
        }

        Ok(packages)
    }

    /// Add an installed package to the database
    pub fn add_package(&mut self, pkg: &InstalledPackage) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO packages
             (category, name, version, slot, installed_at, size, build_time, explicit)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                pkg.id.category,
                pkg.name,
                pkg.version.to_string(),
                pkg.slot,
                pkg.installed_at.to_rfc3339(),
                pkg.size,
                pkg.build_time,
                pkg.explicit,
            ],
        )?;

        let pkg_id = self.conn.last_insert_rowid();

        // Add USE flags
        for flag in &pkg.use_flags {
            self.conn.execute(
                "INSERT INTO package_use_flags (package_id, flag) VALUES (?, ?)",
                params![pkg_id, flag],
            )?;
        }

        // Add files
        for file in &pkg.files {
            self.add_file(pkg_id, file)?;
        }

        Ok(pkg_id)
    }

    /// Remove a package from the database
    pub fn remove_package(&mut self, name: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM packages WHERE name = ?", params![name])?;
        Ok(())
    }

    /// Add a file to a package
    fn add_file(&self, pkg_id: i64, file: &InstalledFile) -> Result<()> {
        self.conn.execute(
            "INSERT INTO files (package_id, path, file_type, mode, size, blake3_hash, mtime)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                pkg_id,
                file.path,
                file.file_type as i32,
                file.mode,
                file.size,
                file.blake3_hash,
                file.mtime,
            ],
        )?;
        Ok(())
    }

    /// Get files for a package by name
    pub fn get_package_files(&self, name: &str) -> Result<Vec<InstalledFile>> {
        let pkg_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM packages WHERE name = ?",
                params![name],
                |row| row.get(0),
            )
            .optional()?;

        match pkg_id {
            Some(id) => self.get_package_files_by_id(id),
            None => Ok(Vec::new()),
        }
    }

    /// Get files for a package by ID
    fn get_package_files_by_id(&self, pkg_id: i64) -> Result<Vec<InstalledFile>> {
        let mut stmt = self.conn.prepare(
            "SELECT path, file_type, mode, size, blake3_hash, mtime
             FROM files WHERE package_id = ?",
        )?;

        let rows = stmt.query_map(params![pkg_id], |row| {
            Ok(InstalledFile {
                path: row.get(0)?,
                file_type: match row.get::<_, i32>(1)? {
                    0 => crate::FileType::Regular,
                    1 => crate::FileType::Directory,
                    2 => crate::FileType::Symlink,
                    3 => crate::FileType::Hardlink,
                    4 => crate::FileType::Device,
                    5 => crate::FileType::Fifo,
                    _ => crate::FileType::Regular,
                },
                mode: row.get(2)?,
                size: row.get(3)?,
                blake3_hash: row.get(4)?,
                mtime: row.get(5)?,
            })
        })?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    /// Get USE flags for a package
    fn get_package_use_flags(&self, pkg_id: i64) -> Result<HashSet<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT flag FROM package_use_flags WHERE package_id = ?")?;

        let rows = stmt.query_map(params![pkg_id], |row| row.get(0))?;

        let mut result = HashSet::new();
        for row in rows {
            result.insert(row?);
        }
        Ok(result)
    }

    /// Get reverse dependencies (packages that depend on this one)
    pub fn get_reverse_dependencies(&self, name: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT p.name FROM packages p
             JOIN dependencies d ON p.id = d.package_id
             WHERE d.dep_name = ?",
        )?;

        let rows = stmt.query_map(params![name], |row| row.get(0))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    /// Add a dependency relationship
    pub fn add_dependency(
        &self,
        pkg_id: i64,
        dep: &PackageId,
        slot: Option<&str>,
        build_time: bool,
        run_time: bool,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO dependencies
             (package_id, dep_category, dep_name, dep_slot, build_time, run_time)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![pkg_id, dep.category, dep.name, slot, build_time, run_time],
        )?;
        Ok(())
    }

    /// Get package that owns a file
    pub fn get_file_owner(&self, path: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT p.name FROM packages p
                 JOIN files f ON p.id = f.package_id
                 WHERE f.path = ?",
                params![path],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.into())
    }

    /// Search installed packages
    pub fn search(&self, query: &str) -> Result<Vec<InstalledPackage>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, category, name, version, slot, installed_at, size, build_time, explicit
             FROM packages WHERE name LIKE ? OR category LIKE ?",
        )?;

        let rows = stmt.query_map(params![&pattern, &pattern], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, u64>(6)?,
                row.get::<_, bool>(7)?,
                row.get::<_, bool>(8)?,
            ))
        })?;

        let mut packages = Vec::new();
        for row in rows {
            let (id, category, name, version, slot, installed_at, size, build_time, explicit) =
                row?;
            let version =
                semver::Version::parse(&version).map_err(|_| Error::InvalidVersion(version))?;
            let installed_at = chrono::DateTime::parse_from_rfc3339(&installed_at)
                .map_err(|e| Error::DatabaseError(e.to_string()))?
                .with_timezone(&chrono::Utc);

            let use_flags = self.get_package_use_flags(id)?;
            let files = self.get_package_files_by_id(id)?;

            packages.push(InstalledPackage {
                id: PackageId::new(category, name.clone()),
                name,
                version,
                slot,
                installed_at,
                use_flags,
                files,
                size,
                build_time,
                explicit,
            });
        }

        Ok(packages)
    }

    /// Begin a transaction
    pub fn begin_transaction(&mut self) -> Result<()> {
        self.conn.execute("BEGIN TRANSACTION", [])?;
        Ok(())
    }

    /// Commit a transaction
    pub fn commit(&mut self) -> Result<()> {
        self.conn.execute("COMMIT", [])?;
        Ok(())
    }

    /// Rollback a transaction
    pub fn rollback(&mut self) -> Result<()> {
        self.conn.execute("ROLLBACK", [])?;
        Ok(())
    }
}
