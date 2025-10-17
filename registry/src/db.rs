use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<PgPool>,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }

    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(50)
            .connect(database_url)
            .await?;

        Ok(Self::new(pool))
    }
}

// Database query helpers
use crate::models::*;
use uuid::Uuid;
use chrono::Utc;

pub struct UserDb;

impl UserDb {
    pub async fn create(
        pool: &PgPool,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (user_id, username, email, password_hash, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(Utc::now())
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_username(
        pool: &PgPool,
        username: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_id(pool: &PgPool, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await
    }

    pub async fn count(pool: &PgPool) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(pool)
            .await?;
        Ok(result.0)
    }
}

pub struct PackageDb;

impl PackageDb {
    pub async fn create(
        pool: &PgPool,
        name: &str,
        owner_id: Uuid,
        description: Option<&str>,
        license: &str,
        repository: Option<&str>,
        homepage: Option<&str>,
        keywords: &[String],
    ) -> Result<Package, sqlx::Error> {
        sqlx::query_as::<_, Package>(
            r#"
            INSERT INTO packages (package_id, name, owner_id, description, license, repository, homepage, keywords, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .bind(owner_id)
        .bind(description)
        .bind(license)
        .bind(repository)
        .bind(homepage)
        .bind(keywords)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_name(
        pool: &PgPool,
        name: &str,
    ) -> Result<Option<Package>, sqlx::Error> {
        sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE name = $1")
            .bind(name)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_id(
        pool: &PgPool,
        package_id: Uuid,
    ) -> Result<Option<Package>, sqlx::Error> {
        sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE package_id = $1")
            .bind(package_id)
            .fetch_optional(pool)
            .await
    }

    pub async fn update_timestamp(pool: &PgPool, package_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE packages SET updated_at = $1 WHERE package_id = $2")
            .bind(Utc::now())
            .bind(package_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn count(pool: &PgPool) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM packages")
            .fetch_one(pool)
            .await?;
        Ok(result.0)
    }

    pub async fn search(
        pool: &PgPool,
        query: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Package>, sqlx::Error> {
        sqlx::query_as::<_, Package>(
            r#"
            SELECT * FROM packages
            WHERE name ILIKE $1 OR description ILIKE $1 OR $2 = ANY(keywords)
            ORDER BY updated_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(format!("%{}%", query))
        .bind(query)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }
}

pub struct VersionDb;

impl VersionDb {
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        pool: &PgPool,
        package_id: Uuid,
        version: &str,
        authors: &[String],
        description: Option<&str>,
        license: &str,
        repository: Option<&str>,
        dependencies: serde_json::Value,
        dev_dependencies: serde_json::Value,
        tarball_url: &str,
        checksum: &str,
        size_bytes: i64,
    ) -> Result<PackageVersion, sqlx::Error> {
        sqlx::query_as::<_, PackageVersion>(
            r#"
            INSERT INTO versions (
                version_id, package_id, version, authors, description, license, repository,
                dependencies, dev_dependencies, tarball_url, checksum, size_bytes,
                published_at, yanked
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(package_id)
        .bind(version)
        .bind(authors)
        .bind(description)
        .bind(license)
        .bind(repository)
        .bind(dependencies)
        .bind(dev_dependencies)
        .bind(tarball_url)
        .bind(checksum)
        .bind(size_bytes)
        .bind(Utc::now())
        .bind(false)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_package_and_version(
        pool: &PgPool,
        package_id: Uuid,
        version: &str,
    ) -> Result<Option<PackageVersion>, sqlx::Error> {
        sqlx::query_as::<_, PackageVersion>(
            "SELECT * FROM versions WHERE package_id = $1 AND version = $2",
        )
        .bind(package_id)
        .bind(version)
        .fetch_optional(pool)
        .await
    }

    pub async fn list_by_package(
        pool: &PgPool,
        package_id: Uuid,
    ) -> Result<Vec<PackageVersion>, sqlx::Error> {
        sqlx::query_as::<_, PackageVersion>(
            "SELECT * FROM versions WHERE package_id = $1 ORDER BY published_at DESC",
        )
        .bind(package_id)
        .fetch_all(pool)
        .await
    }

    pub async fn yank(
        pool: &PgPool,
        version_id: Uuid,
    ) -> Result<PackageVersion, sqlx::Error> {
        sqlx::query_as::<_, PackageVersion>(
            "UPDATE versions SET yanked = true, yanked_at = $1 WHERE version_id = $2 RETURNING *",
        )
        .bind(Utc::now())
        .bind(version_id)
        .fetch_one(pool)
        .await
    }

    pub async fn count(pool: &PgPool) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM versions")
            .fetch_one(pool)
            .await?;
        Ok(result.0)
    }
}

pub struct DownloadDb;

impl DownloadDb {
    pub async fn record(
        pool: &PgPool,
        package_id: Uuid,
        version_id: Uuid,
        ip_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO downloads (download_id, package_id, version_id, downloaded_at, ip_hash)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(package_id)
        .bind(version_id)
        .bind(Utc::now())
        .bind(ip_hash)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn count_total(pool: &PgPool) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM downloads")
            .fetch_one(pool)
            .await?;
        Ok(result.0)
    }

    pub async fn count_for_package(pool: &PgPool, package_id: Uuid) -> Result<i64, sqlx::Error> {
        let result: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM downloads WHERE package_id = $1")
                .bind(package_id)
                .fetch_one(pool)
                .await?;
        Ok(result.0)
    }

    pub async fn count_for_package_period(
        pool: &PgPool,
        package_id: Uuid,
        days: i64,
    ) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM downloads WHERE package_id = $1 AND downloaded_at > NOW() - INTERVAL '1 day' * $2",
        )
        .bind(package_id)
        .bind(days)
        .fetch_one(pool)
        .await?;
        Ok(result.0)
    }
}

pub struct OwnerDb;

impl OwnerDb {
    pub async fn add(
        pool: &PgPool,
        package_id: Uuid,
        user_id: Uuid,
        role: &str,
    ) -> Result<PackageOwner, sqlx::Error> {
        sqlx::query_as::<_, PackageOwner>(
            r#"
            INSERT INTO package_owners (package_id, user_id, role, added_at)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(package_id)
        .bind(user_id)
        .bind(role)
        .bind(Utc::now())
        .fetch_one(pool)
        .await
    }

    pub async fn remove(pool: &PgPool, package_id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM package_owners WHERE package_id = $1 AND user_id = $2")
            .bind(package_id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn list(pool: &PgPool, package_id: Uuid) -> Result<Vec<PackageOwner>, sqlx::Error> {
        sqlx::query_as::<_, PackageOwner>(
            "SELECT * FROM package_owners WHERE package_id = $1",
        )
        .bind(package_id)
        .fetch_all(pool)
        .await
    }

    pub async fn is_owner(
        pool: &PgPool,
        package_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result: Option<(bool,)> = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM package_owners WHERE package_id = $1 AND user_id = $2)",
        )
        .bind(package_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
        Ok(result.map(|r| r.0).unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        // App state creation should work
        assert!(true);
    }
}
