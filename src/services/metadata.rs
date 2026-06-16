use lofty::picture::PictureInformation;
use lofty::prelude::*;
use lofty::probe::Probe;
use std::path::Path;

/// Parsed metadata extracted from an audio file.
pub struct SongMetadata {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub duration_secs: i32,
    pub bit_rate: Option<i32>,
    pub file_format: String,
    pub content_type: String,
    pub year: Option<i32>,
    pub has_cover_art: bool,
}

/// Embedded cover art extracted from an audio file.
pub struct CoverArtData {
    pub mime_type: String,
    pub data: Vec<u8>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

/// Read metadata from an audio file.
///
/// Returns `None` for unreadable files or files without recognized tags.
pub fn read_metadata(path: &Path) -> Option<SongMetadata> {
    let tagged_file = Probe::open(path).ok()?.read().ok()?;

    let properties = tagged_file.properties();
    let tag = tagged_file.primary_tag();

    let title = tag
        .and_then(|t| t.title())
        .map(|s| s.to_string())
        .unwrap_or_default();
    let artist = tag
        .and_then(|t| t.artist())
        .map(|s| s.to_string())
        .unwrap_or_default();
    let album = tag.and_then(|t| t.album()).map(|s| s.to_string());
    let album_artist = tag
        .and_then(|t| t.get_string(ItemKey::AlbumArtist))
        .map(|s| s.to_string());
    let track_number = tag.and_then(|t| t.track()).map(|n| n as i32);
    let disc_number = tag.and_then(|t| t.disk()).map(|n| n as i32);
    let year = tag
        .and_then(|t| {
            t.get_string(ItemKey::Year)
                .or_else(|| t.get_string(ItemKey::RecordingDate))
        })
        .and_then(|s| s.parse::<i32>().ok());

    let duration_secs = properties.duration().as_secs() as i32;
    let bit_rate = properties.audio_bitrate().map(|b| b as i32);

    let has_cover_art = tag.map(|t| !t.pictures().is_empty()).unwrap_or(false);

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let file_format = ext.clone();
    let content_type = mime_type_for_extension(&ext).to_string();

    Some(SongMetadata {
        title,
        artist,
        album,
        album_artist,
        track_number,
        disc_number,
        duration_secs,
        bit_rate,
        file_format,
        content_type,
        year,
        has_cover_art,
    })
}

/// Extract embedded cover art from an audio file.
///
/// Returns `None` if the file has no embedded pictures.
pub fn extract_cover(path: &Path) -> Option<CoverArtData> {
    let tagged_file = Probe::open(path).ok()?.read().ok()?;
    let picture = tagged_file.primary_tag()?.pictures().first()?;

    let mime_type = picture
        .mime_type()
        .map(|mt| mt.to_string())
        .unwrap_or_else(|| "image/jpeg".to_string());
    let data = picture.data().to_vec();

    let (width, height) = PictureInformation::from_picture(picture)
        .ok()
        .map(|info| {
            let w = if info.width > 0 {
                Some(info.width as i32)
            } else {
                None
            };
            let h = if info.height > 0 {
                Some(info.height as i32)
            } else {
                None
            };
            (w, h)
        })
        .unwrap_or((None, None));

    Some(CoverArtData {
        mime_type,
        data,
        width,
        height,
    })
}

/// Map a file extension to its MIME type.
pub fn mime_type_for_extension(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        "ogg" => "audio/ogg",
        "m4a" | "mp4" => "audio/mp4",
        "wav" => "audio/wav",
        "aiff" | "aif" => "audio/aiff",
        "wma" => "audio/x-ms-wma",
        "aac" => "audio/aac",
        "opus" => "audio/opus",
        "wv" => "audio/wavpack",
        "ape" => "audio/ape",
        _ => "application/octet-stream",
    }
}

/// Check whether a file path has a recognised audio extension.
pub fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            matches!(
                e.to_lowercase().as_str(),
                "mp3" | "flac" | "ogg" | "m4a" | "wav" | "aiff" | "wma" | "aac" | "opus" | "wv"
                    | "ape"
            )
        })
        .unwrap_or(false)
}
