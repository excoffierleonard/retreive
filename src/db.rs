use crate::errors::ApiError;
use sqlx::PgPool;

#[derive(Clone)]
pub struct DbPool {
    pool: PgPool,
}

impl DbPool {
    pub async fn new(url: String) -> Result<Self, ApiError> {
        let pool = PgPool::connect(&url).await?;
        Ok(Self { pool })
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }
}
