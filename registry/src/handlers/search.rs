use axum::{extract::{Path, Query, State}, Json};
use crate::{
    db::{AppState, PackageDb, VersionDb, DownloadDb},
    error::AppResult,
    models::*,
};

/// GET /search - Search packages
pub async fn search_packages(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<SearchResponse>> {
    let packages = PackageDb::search(&state.pool, &query.q, query.limit, query.offset).await?;

    let mut results = Vec::new();
    for package in &packages {
        // Get latest version
        let versions = VersionDb::list_by_package(&state.pool, package.package_id).await?;
        let latest_version = versions
            .first()
            .map(|v| v.version.clone())
            .unwrap_or_else(|| "0.0.0".to_string());

        // Get download count
        let downloads = DownloadDb::count_for_package(&state.pool, package.package_id).await?;

        // Calculate relevance score (simple formula)
        let name_match = if package.name.to_lowercase().contains(&query.q.to_lowercase()) {
            2.0
        } else {
            0.0
        };

        let desc_match = if let Some(ref desc) = package.description {
            if desc.to_lowercase().contains(&query.q.to_lowercase()) {
                1.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        let keyword_match = if package.keywords.iter().any(|k| k.to_lowercase() == query.q.to_lowercase()) {
            3.0
        } else {
            0.0
        };

        let download_score = (downloads as f64).log10().max(0.0);
        let score = name_match + desc_match + keyword_match + (download_score * 0.5);

        results.push(SearchResult {
            name: package.name.clone(),
            version: latest_version,
            description: package.description.clone(),
            keywords: package.keywords.clone(),
            downloads,
            score,
        });
    }

    // Sort by relevance score
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    // Count total matching packages (for pagination)
    let total = results.len() as i64;

    Ok(Json(SearchResponse {
        results,
        total,
        limit: query.limit,
        offset: query.offset,
    }))
}

/// GET /packages/trending - Get trending packages
pub async fn trending_packages(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<SearchResult>>> {
    // Get all packages
    let all_packages = PackageDb::search(&state.pool, "", 50, 0).await?;

    let mut results = Vec::new();
    for package in &all_packages {
        // Get latest version
        let versions = VersionDb::list_by_package(&state.pool, package.package_id).await?;
        let latest_version = versions
            .first()
            .map(|v| v.version.clone())
            .unwrap_or_else(|| "0.0.0".to_string());

        // Get download counts (recent vs total)
        let downloads_total = DownloadDb::count_for_package(&state.pool, package.package_id).await?;
        let downloads_week = DownloadDb::count_for_package_period(&state.pool, package.package_id, 7).await?;

        // Calculate trending score based on recent activity
        let recency_days = (chrono::Utc::now() - package.updated_at).num_days().max(1);
        let trend_score = (downloads_week as f64 * 10.0) / (recency_days as f64);

        results.push(SearchResult {
            name: package.name.clone(),
            version: latest_version,
            description: package.description.clone(),
            keywords: package.keywords.clone(),
            downloads: downloads_total,
            score: trend_score,
        });
    }

    // Sort by trending score
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    // Take top 20
    results.truncate(20);

    Ok(Json(results))
}

/// GET /packages/categories/:category - Get packages by category/keyword
pub async fn packages_by_category(
    State(state): State<AppState>,
    Path(category): Path<String>,
) -> AppResult<Json<Vec<SearchResult>>> {
    // Search by keyword (category)
    let packages = PackageDb::search(&state.pool, &category, 100, 0).await?;

    // Filter to only packages that have this keyword
    let filtered: Vec<_> = packages
        .into_iter()
        .filter(|p| {
            p.keywords.iter().any(|k| k.to_lowercase() == category.to_lowercase())
        })
        .collect();

    let mut results = Vec::new();
    for package in &filtered {
        // Get latest version
        let versions = VersionDb::list_by_package(&state.pool, package.package_id).await?;
        let latest_version = versions
            .first()
            .map(|v| v.version.clone())
            .unwrap_or_else(|| "0.0.0".to_string());

        // Get download count
        let downloads = DownloadDb::count_for_package(&state.pool, package.package_id).await?;

        results.push(SearchResult {
            name: package.name.clone(),
            version: latest_version,
            description: package.description.clone(),
            keywords: package.keywords.clone(),
            downloads,
            score: downloads as f64, // Sort by popularity for category pages
        });
    }

    // Sort by downloads (popularity)
    results.sort_by(|a, b| b.downloads.cmp(&a.downloads));

    Ok(Json(results))
}
