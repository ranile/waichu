use common::Asset;
use sqlx::types::Uuid;
use sqlx::PgConnection;
use std::sync::Arc;
use tracing::debug;
use tracing::instrument;

#[instrument]
pub async fn create(db: &mut PgConnection, asset: Asset) -> anyhow::Result<Asset> {
    let Asset { uuid, bytes, .. } = asset;

    debug!("creating asset");

    let asset = sqlx::query!("insert into assets (uuid) values ($1) returning *;", uuid)
        .fetch_one(&mut *db)
        .await?;

    let new_asset = Asset {
        uuid: asset.uuid,
        bytes,
        created_at: asset.created_at,
    };

    Ok(new_asset)
}

#[instrument]
pub async fn get(conn: &mut PgConnection, asset_id: Uuid) -> anyhow::Result<Option<Asset>> {
    debug!("fetching asset");

    let asset = sqlx::query!("select * from assets where uuid = $1;", asset_id)
        .fetch_optional(conn)
        .await?;

    if asset.is_none() {
        debug!("asset {} not found", asset_id);
    }

    Ok(asset.map(|asset| Asset {
        uuid: asset.uuid,
        bytes: Arc::new(vec![]),
        created_at: asset.created_at,
    }))
}

#[instrument]
pub async fn delete(db: &mut PgConnection, asset: &Asset) -> anyhow::Result<()> {
    sqlx::query!("delete from assets where uuid = $1;", asset.uuid)
        .execute(db)
        .await?;
    debug!("deleted asset");
    Ok(())
}

// todo use left joins instead
pub async fn get_from_option(
    conn: &mut PgConnection,
    asset_id: Option<Uuid>,
) -> anyhow::Result<Option<Asset>> {
    let asset = match asset_id {
        Some(id) => get(conn, id).await?,
        None => None,
    };

    Ok(asset)
}
