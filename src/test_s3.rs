use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::Client;
use dotenvy::dotenv;
use sqlx::PgPool;
use std::env;

// Коротко: инициализирует клиентов для S3 и базы данных.
// Конкретно:
// Загружает переменные окружения через .env.
// Читает S3-параметры (endpoint, access key, secret, region, bucket) и DATABASE_URL.
// Создаёт креденшиалы из access/secret.
// Конфигурирует AWS SDK с указанным region, endpoint и провайдером учётных данных.
// Создаёт S3-клиент на основе конфига.
// Подключается к PostgreSQL через sqlx::PgPool.
// Возвращает тройку: (S3-клиент, пул подключений к БД, имя бакета).

pub async fn init_s3_and_db() -> anyhow::Result<(Client, PgPool, String)> {
    dotenv().ok();

    let s3_endpoint   = env::var("S3_ENDPOINT")?;
    let s3_access_key = env::var("S3_ACCESS_KEY")?;
    let s3_secret_key = env::var("S3_SECRET_KEY")?;
    let s3_region     = env::var("S3_REGION")?;
    let s3_bucket     = env::var("S3_BUCKET")?;

    let db_url = env::var("DATABASE_URL")?;

    let creds = Credentials::new(s3_access_key, s3_secret_key, None, None, "static");

    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(s3_region))
        .endpoint_url(s3_endpoint)
        .credentials_provider(creds)
        .load()
        .await;

    let s3_client = Client::new(&config);
    let pool = PgPool::connect(&db_url).await?;

    Ok((s3_client, pool, s3_bucket))
}