#![allow(clippy::toplevel_ref_arg)]
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::error;
#[derive(Debug, Default, sqlx::FromRow, Serialize, Deserialize, Clone, PartialEq)]
pub struct Player {
    pub id: i64,
    pub name: String,
}

#[async_trait]
pub trait PlayerStore {
    async fn insert_player(&self, player: &Player) -> Result<(), sqlx::Error>;
    async fn get_player(&self, id: i64) -> Result<Option<Player>, sqlx::Error>;
}

#[async_trait]
impl PlayerStore for PgPool {
    #[tracing::instrument(name = "Inserting new player", skip(self))]
    async fn insert_player(&self, player: &Player) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO players (id, name) VALUES ($1, $2)",
            player.id,
            player.name
        )
        .execute(self)
        .await
        .map_err(|err| {
            error!("Failed to insert player {}", err);
            err
        })?;
        Ok(())
    }
    #[tracing::instrument(name = "Fetching player", skip(self))]
    async fn get_player(&self, id: i64) -> Result<Option<Player>, sqlx::Error> {
        let player = sqlx::query_as!(Player, "SELECT * FROM players WHERE id = $1", id)
            .fetch_optional(self)
            .await
            .map_err(|err| {
                error!("Failed to get player {}", err);
                err
            })?;
        Ok(player)
    }
}

pub async fn get_or_insert_player(
    player: Player,
    storage: impl PlayerStore,
) -> Result<Player, sqlx::Error> {
    if let Some(player) = storage.get_player(player.id).await? {
        Ok(player)
    } else {
        storage.insert_player(&player).await?;
        Ok(player)
    }
}
