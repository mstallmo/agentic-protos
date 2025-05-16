//! Database module with SQLite support and migration system.
//!
//! This module provides functionality for:
//! - Connection to SQLite database
//! - Applying migrations from the migrations directory
//! - Managing counters (increment, get, set, delete)

use anyhow::{anyhow, Result};
use sqlx::{
    migrate::{Migrator, MigrateDatabase}, 
    sqlite::{SqlitePool, SqlitePoolOptions}, 
    Row, Sqlite
};
use std::{path::Path, sync::Arc};

/// The ID used for the main application counter
pub const MAIN_COUNTER_ID: &str = "main_counter";

/// Database handler for SQLite operations
#[derive(Debug, Clone)]
pub struct Database {
    /// Connection pool for SQLite
    pool: Arc<SqlitePool>,
}

impl Database {
    /// Creates a new Database instance with a connection to SQLite
    /// and applies all pending migrations from the migrations directory.
    ///
    /// # Arguments
    ///
    /// * `database_url` - SQLite connection string (e.g., "sqlite:data.db")
    ///
    /// # Returns
    ///
    /// A Database instance with an initialized connection pool
    pub async fn connect(database_url: &str) -> Result<Self> {
        // Create the database if it doesn't exist
        if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
            println!("Creating new SQLite database at: {}", database_url);
            Sqlite::create_database(database_url).await?;
        } else {
            println!("Connecting to existing SQLite database: {}", database_url);
        }

        // Create a connection pool with reasonable defaults
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
            
        // Create the database instance
        let db = Self {
            pool: Arc::new(pool),
        };

        // Apply migrations and initialize
        db.apply_migrations().await?;
        db.ensure_main_counter().await?;

        Ok(db)
    }

    /// Applies all pending migrations from the migrations directory
    ///
    /// This method will find the migrations directory and apply all
    /// .sql files in sequence based on their names.
    async fn apply_migrations(&self) -> Result<()> {
        println!("Checking for database migrations...");
        
        let migrations_path = Path::new("migrations");
        
        if !migrations_path.exists() {
            return Err(anyhow!("Migrations directory not found at: {}", 
                migrations_path.display()));
        }

        // Load and run migrations from the migrations directory
        let migrator = Migrator::new(migrations_path).await?;
        
        println!("Applying pending migrations from: {}", migrations_path.display());
        migrator.run(&*self.pool).await?;
        
        println!("Database migrations applied successfully");
        Ok(())
    }
    
    /// Ensures the main counter exists in the database
    async fn ensure_main_counter(&self) -> Result<()> {
        // Check if the main counter exists
        let exists = sqlx::query("SELECT 1 FROM counters WHERE id = ?")
            .bind(MAIN_COUNTER_ID)
            .fetch_optional(&*self.pool)
            .await?
            .is_some();
            
        // Create it if it doesn't exist
        if !exists {
            println!("Creating main counter with ID: {}", MAIN_COUNTER_ID);
            sqlx::query("INSERT INTO counters (id, value, description) VALUES (?, 0, ?)")
                .bind(MAIN_COUNTER_ID)
                .bind("Main application counter")
                .execute(&*self.pool)
                .await?;
        }
        
        Ok(())
    }

    /// Gets the value of a counter by ID
    ///
    /// If the counter doesn't exist, it will be created with a value of 0.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the counter to retrieve
    ///
    /// # Returns
    ///
    /// The current value of the counter
    pub async fn get_counter(&self, id: &str) -> Result<i32> {
        let row = sqlx::query("SELECT value FROM counters WHERE id = ?")
            .bind(id)
            .fetch_optional(&*self.pool)
            .await?;

        match row {
            Some(row) => {
                let value: i32 = row.try_get("value")?;
                Ok(value)
            }
            None => {
                // If counter doesn't exist, create it with value 0
                self.set_counter(id, 0).await?;
                Ok(0)
            }
        }
    }

    /// Sets a counter to a specific value
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the counter to set
    /// * `value` - The new value for the counter
    pub async fn set_counter(&self, id: &str, value: i32) -> Result<()> {
        sqlx::query("INSERT OR REPLACE INTO counters (id, value) VALUES (?, ?)")
            .bind(id)
            .bind(value)
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    /// Increments a counter by the specified amount and returns the new value
    ///
    /// This operation is atomic and uses a transaction to ensure consistency
    /// even with concurrent access.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the counter to increment
    /// * `amount` - The amount to increment by
    ///
    /// # Returns
    ///
    /// The new value of the counter after incrementing
    pub async fn increment_counter(&self, id: &str, amount: i32) -> Result<i32> {
        // Start a transaction to ensure atomicity
        let mut tx = self.pool.begin().await?;

        // Get the current counter value or use 0 if it doesn't exist
        let current_value = sqlx::query("SELECT value FROM counters WHERE id = ?")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;

        let current_value = match current_value {
            Some(row) => row.try_get::<i32, _>("value")?,
            None => 0,
        };

        // Calculate the new value
        let new_value = current_value + amount;

        // Update the counter with the new value
        // The highest_value and average_increment will be updated by the trigger
        sqlx::query("INSERT OR REPLACE INTO counters (id, value) VALUES (?, ?)")
            .bind(id)
            .bind(new_value)
            .execute(&mut *tx)
            .await?;

        // Commit the transaction
        tx.commit().await?;

        Ok(new_value)
    }

    /// Lists all counters in the database along with their values
    ///
    /// # Returns
    ///
    /// A vector of (counter_id, value) pairs
    pub async fn list_counters(&self) -> Result<Vec<(String, i32)>> {
        let rows = sqlx::query("SELECT id, value FROM counters ORDER BY id")
            .fetch_all(&*self.pool)
            .await?;

        let mut counters = Vec::with_capacity(rows.len());
        for row in rows {
            let id: String = row.try_get("id")?;
            let value: i32 = row.try_get("value")?;
            counters.push((id, value));
        }

        Ok(counters)
    }

    /// Gets detailed statistics for a counter
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the counter to get statistics for
    ///
    /// # Returns
    ///
    /// A tuple containing (current_value, total_increments, average_increment, highest_value)
    pub async fn get_counter_stats(&self, id: &str) -> Result<Option<(i32, i32, f64, i32)>> {
        let row = sqlx::query(
            "SELECT value, total_increments, average_increment, highest_value 
             FROM counters WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&*self.pool)
        .await?;

        match row {
            Some(row) => {
                let value: i32 = row.try_get("value")?;
                let total_increments: i32 = row.try_get("total_increments")?;
                let average_increment: f64 = row.try_get("average_increment")?;
                let highest_value: i32 = row.try_get("highest_value")?;
                
                Ok(Some((value, total_increments, average_increment, highest_value)))
            }
            None => Ok(None),
        }
    }

    /// Deletes a counter by ID
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the counter to delete
    ///
    /// # Returns
    ///
    /// true if a counter was deleted, false if no counter with that ID existed
    pub async fn delete_counter(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM counters WHERE id = ?")
            .bind(id)
            .execute(&*self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
    
    /// Returns a reference to the underlying connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_counter_operations() -> Result<()> {
        // Use an in-memory database for testing
        let db = Database::connect("sqlite::memory:").await?;

        // Test getting a non-existent counter (should create it and return 0)
        assert_eq!(db.get_counter("test_counter").await?, 0);

        // Test incrementing the counter
        assert_eq!(db.increment_counter("test_counter", 5).await?, 5);

        // Test getting the counter after increment
        assert_eq!(db.get_counter("test_counter").await?, 5);

        // Test incrementing again
        assert_eq!(db.increment_counter("test_counter", 3).await?, 8);

        // Test setting the counter directly
        db.set_counter("test_counter", 10).await?;
        assert_eq!(db.get_counter("test_counter").await?, 10);

        // Test listing counters
        let counters = db.list_counters().await?;
        assert!(counters.len() >= 1);
        
        // The main counter should exist (from ensure_main_counter)
        assert!(counters.iter().any(|(id, _)| id == MAIN_COUNTER_ID));
        
        // Our test counter should exist with value 10
        assert!(counters.iter().any(|(id, val)| id == "test_counter" && *val == 10));

        // Test deleting the counter
        assert!(db.delete_counter("test_counter").await?);

        // After deletion, getting it should create a new one with value 0
        assert_eq!(db.get_counter("test_counter").await?, 0);

        Ok(())
    }
}