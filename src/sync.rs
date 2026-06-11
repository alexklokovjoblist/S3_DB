use aws_sdk_s3::{types::Object, Client};
use sqlx::PgPool;

// Коротко: функция синхронизирует объекты из указанного S3-бакета в таблицу базы данных. Она листит все объекты бакета постранично,
// для каждого объекта берёт ключ (key) и размер, и пытается вставить запись в таблицу files (file_key, bucket, size_bytes, content_type, is_used).
// При конфликте по file_key ничего не делает. Возвращает количество новых вставленных строк.


// Прямая зависимость от sync.rs pub async fn init_s3_and_db(): sync_bucket_to_db требует готовые объекты:
// s3_client: &Client (уже инициализированный S3-клиент)
// pool: &PgPool (уже подключённый пул PostgreSQL)
// bucket: &str (имя бакета)

pub async fn sync_bucket_to_db(
    s3_client: &Client,
    pool: &PgPool,
    bucket: &str,
) -> anyhow::Result<u64> {
    let mut inserted: u64 = 0;

    let mut paginator = s3_client
        .list_objects_v2()
        .bucket(bucket)
        .into_paginator()
        .page_size(1000)
        .send();

    while let Some(page_result) = paginator.next().await {
        let page = page_result?;

        // contents() уже возвращает &[Object], без Option
        let objects: &[Object] = page.contents();

        for obj in objects {
            // key() возвращает Option<&str>
            let key = match obj.key() {
                Some(k) => k.to_owned(), // String
                None => continue,
            };

            let size = obj.size(); // i64

            let content_type: Option<String> = None;

            let res = sqlx::query!(
                r#"
                INSERT INTO files (file_key, bucket, size_bytes, content_type, is_used)
                VALUES ($1, $2, $3, $4, false)
                ON CONFLICT (file_key) DO NOTHING
                "#,
                key,
                bucket,
                size,
                content_type
            )
            .execute(pool)
            .await?;

            if res.rows_affected() > 0 {
                inserted += 1;
            }
        }
    }

    Ok(inserted)
}