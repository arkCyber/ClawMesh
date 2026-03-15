//! Thumbnail generation for images and videos

use anyhow::Result;

/// Thumbnail generator
pub struct ThumbnailGenerator {
    /// Target width
    width: u32,
    /// Target height
    height: u32,
}

impl ThumbnailGenerator {
    /// Create a new thumbnail generator
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Generate thumbnail from image data
    ///
    /// # Errors
    /// Returns error if thumbnail generation fails
    pub async fn generate_image_thumbnail(&self, _image_data: &[u8]) -> Result<Vec<u8>> {
        // TODO: Implement actual image thumbnail generation
        // This would use image processing library like image-rs
        Err(anyhow::anyhow!("Not implemented"))
    }

    /// Generate thumbnail from video data
    ///
    /// # Errors
    /// Returns error if thumbnail generation fails
    pub async fn generate_video_thumbnail(&self, _video_data: &[u8]) -> Result<Vec<u8>> {
        // TODO: Implement actual video thumbnail generation
        // This would use ffmpeg or similar
        Err(anyhow::anyhow!("Not implemented"))
    }

    /// Calculate thumbnail dimensions maintaining aspect ratio
    #[must_use]
    pub fn calculate_dimensions(&self, original_width: u32, original_height: u32) -> (u32, u32) {
        let aspect_ratio = original_width as f32 / original_height as f32;
        let target_aspect = self.width as f32 / self.height as f32;

        if aspect_ratio > target_aspect {
            // Width is the limiting factor
            let new_width = self.width;
            let new_height = (self.width as f32 / aspect_ratio) as u32;
            (new_width, new_height)
        } else {
            // Height is the limiting factor
            let new_height = self.height;
            let new_width = (self.height as f32 * aspect_ratio) as u32;
            (new_width, new_height)
        }
    }
}

impl Default for ThumbnailGenerator {
    fn default() -> Self {
        Self::new(300, 300)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_dimensions_landscape() {
        let generator = ThumbnailGenerator::new(300, 300);
        let (width, height) = generator.calculate_dimensions(1920, 1080);
        
        assert_eq!(width, 300);
        assert!(height < 300);
        assert!(height > 0);
    }

    #[test]
    fn test_calculate_dimensions_portrait() {
        let generator = ThumbnailGenerator::new(300, 300);
        let (width, height) = generator.calculate_dimensions(1080, 1920);
        
        assert!(width < 300);
        assert_eq!(height, 300);
        assert!(width > 0);
    }

    #[test]
    fn test_calculate_dimensions_square() {
        let generator = ThumbnailGenerator::new(300, 300);
        let (width, height) = generator.calculate_dimensions(1000, 1000);
        
        assert_eq!(width, 300);
        assert_eq!(height, 300);
    }
}
