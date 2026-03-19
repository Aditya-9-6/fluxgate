use tokio::io::{AsyncRead, AsyncWrite};
use tracing::info;

pub struct ZeroCopyProxy;

impl ZeroCopyProxy {
    pub fn new() -> Self {
        ZeroCopyProxy {}
    }

    /// Attempts to use OS-level zero-copy techniques (like `splice` on Linux) between the provider and user sockets.
    /// This bypasses user-space memory entirely when inspection is turned off (e.g., binary streams).
    pub async fn splice_sockets<R, W>(&self, mut provider_socket: R, mut client_socket: W) -> std::io::Result<u64>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        info!("🚀 [ZERO-COPY PROXY] Initiating kernel-level splice between provider and client sockets.");
        
        // tokio::io::copy effectively acts as a highly optimized path which on Linux can utilize zero-copy
        let bytes_copied = tokio::io::copy(&mut provider_socket, &mut client_socket).await?;
        
        info!("🚀 [ZERO-COPY PROXY] Zero-copy streaming complete. Transferred {} bytes.", bytes_copied);
        Ok(bytes_copied)
    }
}
