// src/s3_client.rs
use aws_sdk_s3::Client;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;

#[derive(Clone)]
pub struct S3Config {
    pub bucket: String,
    pub key: String,
    pub local_path: String,
}

pub async fn ensure_db_exists(client: &Client, config: &S3Config) -> Result<(), String> {
    if Path::new(&config.local_path).exists() {
        println!("Database found locally on {}", config.local_path);
        return Ok(());
    }

    println!("Database missing. Downloading from S3...");
    
    let resp = client
        .get_object()
        .bucket(&config.bucket)
        .key(&config.key)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch from S3: {}", e))?;

    let data = resp.body.collect().await.map_err(|e| e.to_string())?;
    let mut file = File::create(&config.local_path).await.map_err(|e| e.to_string())?;
    file.write_all(&data.into_bytes()).await.map_err(|e| e.to_string())?;

    println!("Database downloaded successfully.");
    Ok(())
}

pub async fn upload_db(client: &Client, config: &S3Config) -> Result<(), String> {
    println!("Uploading database to S3...");
    
    let mut file = File::open(&config.local_path).await.map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await.map_err(|e| e.to_string())?;

    client
        .put_object()
        .bucket(&config.bucket)
        .key(&config.key)
        .body(buffer.into())
        .send()
        .await
        .map_err(|e| format!("Failed to upload to S3: {}", e))?;

    println!("Database uploaded successfully.");
    Ok(())
}