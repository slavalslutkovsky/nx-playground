use rpc::tasks::tasks_service_client::TasksServiceClient;
use tonic::transport::{Channel, Endpoint};
use std::time::Duration;

/// Creates an optimized gRPC client with HTTP/2 tuning and compression
pub async fn create_optimized_tasks_client(addr: String) -> eyre::Result<TasksServiceClient<Channel>> {
    let endpoint = Endpoint::from_shared(addr)?
        // HTTP/2 settings for high throughput
        .http2_keep_alive_interval(Duration::from_secs(30))
        .keep_alive_timeout(Duration::from_secs(10))
        .keep_alive_while_idle(true)
        // Connection settings
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(30))
        // Allow more concurrent streams per connection
        .initial_connection_window_size(1024 * 1024) // 1MB
        .initial_stream_window_size(1024 * 1024)     // 1MB
        // Adaptive flow control for better throughput
        .http2_adaptive_window(true)
        // TCP tuning
        .tcp_nodelay(true)
        .tcp_keepalive(Some(Duration::from_secs(30)));

    let channel = endpoint.connect().await?;

    let client = TasksServiceClient::new(channel)
        // Using zstd for better performance (3-5x faster than gzip)
        // Add to Cargo.toml: tonic = { version = "0.12", features = ["zstd"] }
        .accept_compressed(tonic::codec::CompressionEncoding::Zstd)
        .send_compressed(tonic::codec::CompressionEncoding::Zstd)
        .max_decoding_message_size(8 * 1024 * 1024)  // 8MB max
        .max_encoding_message_size(8 * 1024 * 1024); // 8MB max

    Ok(client)
}

/// Creates a pool of gRPC clients for better concurrency
pub async fn create_client_pool(addr: String, pool_size: usize) -> eyre::Result<Vec<TasksServiceClient<Channel>>> {
    let mut clients = Vec::with_capacity(pool_size);

    for _ in 0..pool_size {
        let client = create_optimized_tasks_client(addr.clone()).await?;
        clients.push(client);
    }

    Ok(clients)
}
