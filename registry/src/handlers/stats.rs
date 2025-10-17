use axum::{extract::{Path, State}, Json};
use chrono::Utc;
use crate::{
    db::{AppState, PackageDb, UserDb, VersionDb, DownloadDb},
    error::AppResult,
    models::{GlobalStats, PackageStats, DownloadStats},
};

/// GET /stats - Get global registry statistics
pub async fn global_stats(State(state): State<AppState>) -> AppResult<Json<GlobalStats>> {
    let total_packages = PackageDb::count(&state.pool).await?;
    let total_versions = VersionDb::count(&state.pool).await?;
    let total_downloads = DownloadDb::count_total(&state.pool).await?;
    let total_users = UserDb::count(&state.pool).await?;

    Ok(Json(GlobalStats {
        total_packages,
        total_versions,
        total_downloads,
        total_users,
        updated_at: Utc::now(),
    }))
}

/// GET /packages/:name/stats - Get package-specific statistics
pub async fn package_stats(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> AppResult<Json<PackageStats>> {
    // Find package
    let package = PackageDb::find_by_name(&state.pool, &name)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("Package not found".to_string()))?;

    // Get download stats
    let total = DownloadDb::count_for_package(&state.pool, package.package_id).await?;
    let last_week = DownloadDb::count_for_package_period(&state.pool, package.package_id, 7).await?;
    let last_month = DownloadDb::count_for_package_period(&state.pool, package.package_id, 30).await?;
    let last_year = DownloadDb::count_for_package_period(&state.pool, package.package_id, 365).await?;

    // Get versions count
    let versions = VersionDb::list_by_package(&state.pool, package.package_id).await?;
    let versions_count = versions.len() as i64;

    Ok(Json(PackageStats {
        name: package.name,
        downloads: DownloadStats {
            total,
            last_week,
            last_month,
            last_year,
        },
        versions_count,
        dependents_count: 0, // TODO: Implement dependents tracking
        updated_at: Utc::now(),
    }))
}
