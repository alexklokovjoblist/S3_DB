// src/main.rs
mod test_s3;
mod sync;
mod utils;



#[tokio::main]
async fn main() -> anyhow::Result<()> {
  
    // Инициализируем S3 + DB один раз
    let (s3_client, pool, bucket) = test_s3::init_s3_and_db().await?;

    // Синхронизация бакета в БД
    let inserted = sync::sync_bucket_to_db(&s3_client, &pool, &bucket).await?;
    println!("Sync done. Inserted {} new records.", inserted);
    
    //демонстрация модульности
    utils::help();
    Ok(())
}