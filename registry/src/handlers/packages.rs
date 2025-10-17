use axum::{
    body::Bytes,
    extract::{Path, State, Multipart},
    http::StatusCode,
    Json,
};
use serde_json::json;
use sha2::{Sha256, Digest};
use uuid::Uuid;
use std::io::Write as _;

use crate::{
    auth::AuthUser,
    db::{AppState, PackageDb, VersionDb, DownloadDb, OwnerDb},
    error::{AppError, AppResult},
    models::*,
    validation::validate,
};

/// POST /packages/publish - Publish a new package version
pub async fn publish(
    State(state): State<AppState>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> AppResult<(StatusCode, Json<PublishResponse>)> {
    let mut metadata: Option<PublishRequest> = None;
    let mut tarball: Option<Bytes> = None;

    // Parse multipart form
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();

        match name.as_str() {
            "metadata" => {
                let data = field.bytes().await.unwrap();
                metadata = Some(serde_json::from_slice(&data)?);
            }
            "tarball" => {
                tarball = Some(field.bytes().await.unwrap());
            }
            _ => {}
        }
    }

    let metadata = metadata.ok_or(AppError::BadRequest("Missing metadata".to_string()))?;
    let tarball = tarball.ok_or(AppError::BadRequest("Missing tarball".to_string()))?;

    // Validate metadata
    validate(&metadata)?;

    // Calculate checksum
    let mut hasher = Sha256::new();
    hasher.update(&tarball);
    let checksum = format!("{:x}", hasher.finalize());

    // Check if package exists
    let package = PackageDb::find_by_name(&state.pool, &metadata.name).await?;

    let package = if let Some(existing) = package {
        // Check if user is owner
        if !OwnerDb::is_owner(&state.pool, existing.package_id, auth.user_id).await? {
            return Err(AppError::Forbidden(
                "You are not an owner of this package".to_string(),
            ));
        }

        // Check if version already exists
        if VersionDb::find_by_package_and_version(&state.pool, existing.package_id, &metadata.version)
            .await?
            .is_some()
        {
            return Err(AppError::Conflict(format!(
                "Version {} already exists",
                metadata.version
            )));
        }

        existing
    } else {
        // Create new package
        let new_package = PackageDb::create(
            &state.pool,
            &metadata.name,
            auth.user_id,
            metadata.description.as_deref(),
            &metadata.license,
            metadata.repository.as_deref(),
            metadata.homepage.as_deref(),
            &metadata.keywords,
        )
        .await?;

        // Add creator as owner
        OwnerDb::add(&state.pool, new_package.package_id, auth.user_id, "owner").await?;

        new_package
    };

    // Save tarball to storage
    let tarball_path = format!("packages/{}/{}.tar.gz", metadata.name, metadata.version);
    let storage_dir = std::env::var("STORAGE_DIR").unwrap_or_else(|_| "./storage".to_string());
    std::fs::create_dir_all(format!("{}/packages/{}", storage_dir, metadata.name))?;

    let full_path = format!("{}/{}", storage_dir, tarball_path);
    let mut file = std::fs::File::create(&full_path)?;
    file.write_all(&tarball)?;

    // Create version record
    let tarball_url = format!("/api/v1/packages/{}/{}/download", metadata.name, metadata.version);
    let dependencies = serde_json::to_value(&metadata.dependencies)?;
    let dev_dependencies = serde_json::to_value(&metadata.dev_dependencies)?;

    let version = VersionDb::create(
        &state.pool,
        package.package_id,
        &metadata.version,
        &metadata.authors,
        metadata.description.as_deref(),
        &metadata.license,
        metadata.repository.as_deref(),
        dependencies,
        dev_dependencies,
        &tarball_url,
        &checksum,
        tarball.len() as i64,
    )
    .await?;

    // Update package timestamp
    PackageDb::update_timestamp(&state.pool, package.package_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(PublishResponse {
            package_id: package.package_id,
            name: metadata.name.clone(),
            version: metadata.version.clone(),
            published_at: version.published_at,
            download_url: tarball_url.clone(),
            checksum,
        }),
    ))
}

/// GET /packages/:name - Get package metadata
pub async fn get_package(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> AppResult<Json<PackageResponse>> {
    let package = PackageDb::find_by_name(&state.pool, &name)
        .await?
        .ok_or(AppError::NotFound(format!("Package '{}' not found", name)))?;

    // Get owner info
    let owner = crate::db::UserDb::find_by_id(&state.pool, package.owner_id)
        .await?
        .ok_or(AppError::NotFound("Owner not found".to_string()))?;

    // Get all versions
    let versions = VersionDb::list_by_package(&state.pool, package.package_id).await?;
    let version_strings: Vec<String> = versions.iter().map(|v| v.version.clone()).collect();
    let latest_version = versions
        .first()
        .map(|v| v.version.clone())
        .unwrap_or_else(|| "0.0.0".to_string());

    // Get download counts
    let downloads_total = DownloadDb::count_for_package(&state.pool, package.package_id).await?;
    let downloads_last_month =
        DownloadDb::count_for_package_period(&state.pool, package.package_id, 30).await?;

    Ok(Json(PackageResponse {
        name: package.name,
        description: package.description,
        latest_version,
        versions: version_strings,
        owner: OwnerInfo {
            username: owner.username,
            user_id: owner.user_id,
        },
        license: package.license,
        repository: package.repository,
        homepage: package.homepage,
        keywords: package.keywords,
        downloads_total,
        downloads_last_month,
        created_at: package.created_at,
        updated_at: package.updated_at,
    }))
}

/// GET /packages/:name/:version - Get specific version metadata
pub async fn get_version(
    State(state): State<AppState>,
    Path((name, version_str)): Path<(String, String)>,
) -> AppResult<Json<VersionResponse>> {
    let package = PackageDb::find_by_name(&state.pool, &name)
        .await?
        .ok_or(AppError::NotFound(format!("Package '{}' not found", name)))?;

    let version = VersionDb::find_by_package_and_version(&state.pool, package.package_id, &version_str)
        .await?
        .ok_or(AppError::NotFound(format!(
            "Version '{}' not found for package '{}'",
            version_str, name
        )))?;

    Ok(Json(VersionResponse {
        name: package.name,
        version: version.version,
        description: version.description,
        authors: version.authors,
        license: version.license,
        repository: version.repository,
        dependencies: version.dependencies,
        dev_dependencies: version.dev_dependencies,
        published_at: version.published_at,
        download_url: version.tarball_url.clone(),
        checksum: version.checksum,
        size_bytes: version.size_bytes,
        yanked: version.yanked,
    }))
}

/// GET /packages/:name/:version/download - Download package tarball
pub async fn download(
    State(state): State<AppState>,
    Path((name, version_str)): Path<(String, String)>,
) -> AppResult<Bytes> {
    let package = PackageDb::find_by_name(&state.pool, &name)
        .await?
        .ok_or(AppError::NotFound(format!("Package '{}' not found", name)))?;

    let version = VersionDb::find_by_package_and_version(&state.pool, package.package_id, &version_str)
        .await?
        .ok_or(AppError::NotFound(format!(
            "Version '{}' not found",
            version_str
        )))?;

    if version.yanked {
        return Err(AppError::BadRequest(format!(
            "Version {} has been yanked",
            version_str
        )));
    }

    // Read tarball from storage
    let storage_dir = std::env::var("STORAGE_DIR").unwrap_or_else(|_| "./storage".to_string());
    let tarball_path = format!("{}/packages/{}/{}.tar.gz", storage_dir, name, version_str);

    let data = std::fs::read(&tarball_path).map_err(|e| {
        AppError::InternalServerError(format!("Failed to read tarball: {}", e))
    })?;

    // Record download (with placeholder IP hash)
    let ip_hash = "placeholder"; // In production, hash the actual IP
    DownloadDb::record(&state.pool, package.package_id, version.version_id, ip_hash).await?;

    Ok(Bytes::from(data))
}

/// DELETE /packages/:name/:version - Yank a version
pub async fn yank_version(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((name, version_str)): Path<(String, String)>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    let package = PackageDb::find_by_name(&state.pool, &name)
        .await?
        .ok_or(AppError::NotFound(format!("Package '{}' not found", name)))?;

    // Check if user is owner
    if !OwnerDb::is_owner(&state.pool, package.package_id, auth.user_id).await? {
        return Err(AppError::Forbidden(
            "You are not an owner of this package".to_string(),
        ));
    }

    let version = VersionDb::find_by_package_and_version(&state.pool, package.package_id, &version_str)
        .await?
        .ok_or(AppError::NotFound(format!(
            "Version '{}' not found",
            version_str
        )))?;

    if version.yanked {
        return Err(AppError::BadRequest(format!(
            "Version {} is already yanked",
            version_str
        )));
    }

    // Yank the version
    VersionDb::yank(&state.pool, version.version_id).await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": format!("Version {} has been yanked", version_str),
            "package": name,
            "version": version_str
        })),
    ))
}

/// GET /packages/:name/owners - List package owners
pub async fn list_owners(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> AppResult<Json<Vec<OwnerResponse>>> {
    let package = PackageDb::find_by_name(&state.pool, &name)
        .await?
        .ok_or(AppError::NotFound(format!("Package '{}' not found", name)))?;

    let owners = OwnerDb::list(&state.pool, package.package_id).await?;

    let mut owner_responses = Vec::new();
    for owner in owners {
        let user = crate::db::UserDb::find_by_id(&state.pool, owner.user_id)
            .await?
            .ok_or(AppError::NotFound("User not found".to_string()))?;

        owner_responses.push(OwnerResponse {
            username: user.username,
            user_id: user.user_id,
            role: owner.role,
        });
    }

    Ok(Json(owner_responses))
}

/// PUT /packages/:name/owners - Add a package owner
pub async fn add_owner(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(name): Path<String>,
    Json(req): Json<AddOwnerRequest>,
) -> AppResult<Json<OwnerResponse>> {
    let package = PackageDb::find_by_name(&state.pool, &name)
        .await?
        .ok_or(AppError::NotFound(format!("Package '{}' not found", name)))?;

    // Check if requester is owner
    if !OwnerDb::is_owner(&state.pool, package.package_id, auth.user_id).await? {
        return Err(AppError::Forbidden(
            "You are not an owner of this package".to_string(),
        ));
    }

    // Find user to add
    let user_to_add = crate::db::UserDb::find_by_username(&state.pool, &req.username)
        .await?
        .ok_or(AppError::NotFound(format!(
            "User '{}' not found",
            req.username
        )))?;

    // Check if already owner
    if OwnerDb::is_owner(&state.pool, package.package_id, user_to_add.user_id).await? {
        return Err(AppError::Conflict(format!(
            "{} is already an owner",
            req.username
        )));
    }

    // Add owner
    OwnerDb::add(&state.pool, package.package_id, user_to_add.user_id, &req.role).await?;

    Ok(Json(OwnerResponse {
        username: user_to_add.username,
        user_id: user_to_add.user_id,
        role: req.role,
    }))
}

/// DELETE /packages/:name/owners/:username - Remove a package owner
pub async fn remove_owner(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((name, username)): Path<(String, String)>,
) -> AppResult<Json<serde_json::Value>> {
    let package = PackageDb::find_by_name(&state.pool, &name)
        .await?
        .ok_or(AppError::NotFound(format!("Package '{}' not found", name)))?;

    // Check if requester is owner
    if !OwnerDb::is_owner(&state.pool, package.package_id, auth.user_id).await? {
        return Err(AppError::Forbidden(
            "You are not an owner of this package".to_string(),
        ));
    }

    // Find user to remove
    let user_to_remove = crate::db::UserDb::find_by_username(&state.pool, &username)
        .await?
        .ok_or(AppError::NotFound(format!("User '{}' not found", username)))?;

    // Prevent removing yourself if you're the only owner
    let owners = OwnerDb::list(&state.pool, package.package_id).await?;
    if owners.len() == 1 && user_to_remove.user_id == auth.user_id {
        return Err(AppError::BadRequest(
            "Cannot remove yourself as the only owner".to_string(),
        ));
    }

    // Remove owner
    OwnerDb::remove(&state.pool, package.package_id, user_to_remove.user_id).await?;

    Ok(Json(json!({
        "message": format!("{} has been removed as an owner", username),
        "package": name,
        "username": username
    })))
}
