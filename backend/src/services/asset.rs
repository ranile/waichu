use crate::services::optional_value_or_err;
use common::Asset;
use sqlx::types::Uuid;
use sqlx::PgConnection;
use std::sync::Arc;

macro_rules! construct_asset {
    ($asset:ident) => {{
        match $asset {
            Ok(asset) => Ok(Asset {
                uuid: asset.uuid,
                bytes: Arc::new(vec![]),
                created_at: asset.created_at,
            }),
            Err(e) => Err(e),
        }
    }};
}

pub async fn create(db: &mut PgConnection, asset: Asset) -> anyhow::Result<Asset> {
    let Asset { uuid, bytes, .. } = asset;

    let inserted = sqlx::query!("insert into assets (uuid) values ($1) returning *;", uuid)
        .fetch_one(&mut *db)
        .await;

    let mut new_asset = construct_asset!(inserted)?;
    new_asset.bytes = bytes;

    println!("new asset bytes {}", new_asset.bytes.len());

    Ok(new_asset)
}

pub async fn get(conn: &mut PgConnection, asset_id: Uuid) -> anyhow::Result<Option<Asset>> {
    let asset = sqlx::query!("select * from assets where uuid = $1;", asset_id)
        .fetch_one(conn)
        .await;

    optional_value_or_err(construct_asset!(asset))
}

pub async fn delete(db: &mut PgConnection, asset: &Asset) -> anyhow::Result<()> {
    sqlx::query!("delete from assets where uuid = $1;", asset.uuid)
        .execute(db)
        .await?;
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
