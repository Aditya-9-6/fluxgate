pub mod compressor;
pub mod stream_stitcher;
pub mod mutation;
pub mod distiller;
pub mod distiller_v2;
pub struct UnifiedResponseTransformer;

impl UnifiedResponseTransformer {
    pub fn new() -> Self { Self }

    pub fn transform_response(&self, provider: &str, text: &str) -> Option<String> {
        Some(text.to_string())
    }

    pub fn transform_stream_chunk(&self, provider: &str, text: &str) -> String {
        text.to_string()
    }
}
