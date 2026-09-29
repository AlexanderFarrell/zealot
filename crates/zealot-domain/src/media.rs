use std::{
    fs::Metadata,
    path::{Component, Path, PathBuf},
};

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use size::Size;

// Domain

#[derive(Debug, Clone)]
pub struct FileStat {
    path: PathBuf,
    size: Size,
    is_folder: bool,
    modified_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct File {
    pub stat: FileStat,
    pub bytes: Vec<u8>,
}

// Send DTOs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatDto {
    pub path: String,
    pub size: i64,
    pub is_folder: bool,
    pub modified_at: i64,
}

// Receive DTOs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakeFolderDto {
    pub folder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameMediaDto {
    pub old_location: String,
    pub new_name: String,
}

// Errors

#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("error getting modified time: {err}")]
    ModifiedTimeError { err: std::io::Error },

    #[error("invalid modified timestamp: {timestamp}")]
    InvalidModifiedTimestamp { timestamp: i64 },

    #[error("file path must be relative: {path}")]
    AbsolutePathNotAllowed { path: String },

    #[error("file path cannot contain parent directory traversal: {path}")]
    ParentDirNotAllowed { path: String },

    #[error("file path cannot be empty")]
    EmptyPathNotAllowed,
}

// Impls

impl FileStat {
    pub fn new<P: AsRef<Path>>(
        path: P,
        size: Size,
        is_folder: bool,
        modified_at: NaiveDateTime,
    ) -> Result<Self, MediaError> {
        Ok(Self {
            path: sanitize_relative_path(path)?,
            size,
            is_folder,
            modified_at,
        })
    }

    pub fn from_metadata<P: AsRef<Path>>(path: P, metadata: &Metadata) -> Result<Self, MediaError> {
        let modified_at = metadata
            .modified()
            .map(|time| DateTime::<Utc>::from(time).naive_utc())
            .map_err(|err| MediaError::ModifiedTimeError { err })?;

        Self::new(
            path,
            Size::from_bytes(metadata.len()),
            metadata.is_dir(),
            modified_at,
        )
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn is_folder(&self) -> bool {
        self.is_folder
    }

    pub fn modified_at(&self) -> NaiveDateTime {
        self.modified_at
    }

    pub fn path_str(&self) -> String {
        path_to_dto_string(self.path())
    }

    pub fn size_bytes(&self) -> i64 {
        self.size.bytes()
    }

    pub fn name(&self) -> String {
        self.path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.path_str())
    }

    pub fn extension(&self) -> String {
        if self.is_folder {
            return String::new();
        }

        self.path
            .extension()
            .map(|extension| format!(".{}", extension.to_string_lossy().to_lowercase()))
            .unwrap_or_default()
    }

    pub fn icon(&self) -> &'static str {
        if self.is_folder {
            return "📁";
        }

        file_icon(self.extension().as_str())
    }

    pub fn calculate_etag(&self) -> String {
        todo!()
    }

    pub fn type_description(&self) -> &'static str {
        if self.is_folder {
            return "Folder";
        }

        file_type_description(self.extension().as_str())
    }

    pub fn modified_date_str(&self) -> String {
        self.modified_at.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn display_size(&self) -> String {
        let bytes = self.size_bytes();

        if bytes < 1024 {
            return format!("{} B", bytes);
        }

        let units = ["KB", "MB", "GB", "TB"];
        let mut value = bytes as f64 / 1024.0;
        let mut unit_index = 0;

        while value >= 1024.0 && unit_index < units.len() - 1 {
            value /= 1024.0;
            unit_index += 1;
        }

        let decimals = if value >= 10.0 { 0 } else { 1 };
        format!("{value:.decimals$} {}", units[unit_index])
    }

    pub fn is_image(&self) -> bool {
        matches!(
            self.extension().as_str(),
            ".png" | ".jpg" | ".jpeg" | ".gif" | ".webp" | ".tga" | ".svg" | ".avif"
        )
    }

    pub fn is_video(&self) -> bool {
        matches!(
            self.extension().as_str(),
            ".mp4" | ".mov" | ".mkv" | ".webm" | ".avi"
        )
    }

    pub fn is_audio(&self) -> bool {
        matches!(
            self.extension().as_str(),
            ".mp3" | ".wav" | ".aac" | ".m4a" | ".ogg" | ".flac"
        )
    }
}

impl TryFrom<FileStatDto> for FileStat {
    type Error = MediaError;

    fn try_from(dto: FileStatDto) -> Result<Self, Self::Error> {
        let modified_at = DateTime::<Utc>::from_timestamp(dto.modified_at, 0)
            .map(|timestamp| timestamp.naive_utc())
            .ok_or(MediaError::InvalidModifiedTimestamp {
                timestamp: dto.modified_at,
            })?;

        Self::new(
            dto.path,
            Size::from_bytes(dto.size),
            dto.is_folder,
            modified_at,
        )
    }
}

impl From<&FileStat> for FileStatDto {
    fn from(value: &FileStat) -> Self {
        Self {
            path: value.path_str(),
            size: value.size_bytes(),
            is_folder: value.is_folder,
            modified_at: value.modified_at.and_utc().timestamp(),
        }
    }
}

impl From<FileStat> for FileStatDto {
    fn from(value: FileStat) -> Self {
        Self::from(&value)
    }
}

fn path_to_dto_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn sanitize_relative_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, MediaError> {
    let path = path.as_ref();
    let raw_path = path_to_dto_string(path);
    let mut sanitized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Normal(part) => sanitized.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(MediaError::ParentDirNotAllowed { path: raw_path });
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(MediaError::AbsolutePathNotAllowed { path: raw_path });
            }
        }
    }

    if sanitized.as_os_str().is_empty() {
        return Err(MediaError::EmptyPathNotAllowed);
    }

    Ok(sanitized)
}

fn file_icon(extension: &str) -> &'static str {
    match extension {
        ".txt" | ".md" | ".rtf" | ".pdf" | ".doc" | ".docx" | ".odt" => "📄",
        ".csv" | ".tsv" | ".xls" | ".xlsx" | ".ods" | ".ppt" | ".pptx" | ".odp" => "📊",
        ".png" | ".jpg" | ".jpeg" | ".gif" | ".bmp" | ".webp" | ".tiff" | ".svg" | ".ico"
        | ".heic" | ".avif" => "🖼️",
        ".mp4" | ".mov" | ".mkv" | ".webm" | ".avi" | ".m4v" | ".wmv" => "🎬",
        ".mp3" | ".wav" | ".aac" | ".m4a" | ".flac" | ".ogg" | ".opus" => "🎵",
        ".zip" | ".rar" | ".7z" | ".tar" | ".gz" | ".bz2" | ".xz" => "🗜️",
        ".epub" | ".mobi" => "📕",
        ".js" | ".ts" | ".jsx" | ".tsx" | ".json" | ".html" | ".css" | ".scss" | ".go" | ".py"
        | ".rb" | ".php" | ".java" | ".c" | ".cpp" | ".rs" | ".sh" | ".sql" | ".yml" | ".yaml" => {
            "💻"
        }
        ".psd" | ".ai" | ".fig" | ".sketch" => "🎨",
        _ => "📄",
    }
}

fn file_type_description(extension: &str) -> &'static str {
    match extension {
        ".txt" => "Text file",
        ".md" => "Markdown document",
        ".rtf" => "Rich Text document",
        ".pdf" => "PDF document",
        ".doc" | ".docx" => "Word document",
        ".odt" => "OpenDocument text",
        ".csv" => "Comma Separated Values file",
        ".tsv" => "Tab Separated Values file",
        ".xls" | ".xlsx" => "Excel spreadsheet",
        ".ods" => "OpenDocument spreadsheet",
        ".ppt" | ".pptx" => "PowerPoint presentation",
        ".odp" => "OpenDocument presentation",
        ".png" => "PNG image",
        ".jpg" | ".jpeg" => "JPEG image",
        ".gif" => "GIF image",
        ".bmp" => "Bitmap image",
        ".webp" => "WebP image",
        ".tiff" => "TIFF image",
        ".svg" => "SVG image",
        ".ico" => "Icon image",
        ".heic" => "HEIC image",
        ".avif" => "AVIF image",
        ".mp4" => "MP4 video",
        ".mov" => "QuickTime video",
        ".mkv" => "Matroska video",
        ".webm" => "WebM video",
        ".avi" => "AVI video",
        ".m4v" => "M4V video",
        ".wmv" => "WMV video",
        ".mp3" => "MP3 audio",
        ".wav" => "WAV audio",
        ".aac" => "AAC audio",
        ".m4a" => "M4A audio",
        ".flac" => "FLAC audio",
        ".ogg" => "Ogg audio",
        ".opus" => "Opus audio",
        ".zip" => "ZIP archive",
        ".rar" => "RAR archive",
        ".7z" => "7-Zip archive",
        ".tar" => "TAR archive",
        ".gz" => "Gzip archive",
        ".bz2" => "Bzip2 archive",
        ".xz" => "XZ archive",
        ".epub" => "EPUB book",
        ".mobi" => "MOBI book",
        ".js" => "JavaScript file",
        ".ts" => "TypeScript file",
        ".jsx" => "JSX file",
        ".tsx" => "TSX file",
        ".json" => "JSON file",
        ".html" => "HTML document",
        ".css" => "CSS stylesheet",
        ".scss" => "SCSS stylesheet",
        ".go" => "Go source file",
        ".py" => "Python file",
        ".rb" => "Ruby file",
        ".php" => "PHP file",
        ".java" => "Java source file",
        ".c" => "C source file",
        ".cpp" => "C++ source file",
        ".rs" => "Rust source file",
        ".sh" => "Shell script",
        ".sql" => "SQL file",
        ".yml" | ".yaml" => "YAML file",
        ".psd" => "Photoshop document",
        ".ai" => "Illustrator document",
        ".fig" => "Figma design",
        ".sketch" => "Sketch design",
        _ => "File",
    }
}

#[cfg(test)]
mod media_tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    fn test_timestamp() -> NaiveDateTime {
        DateTime::<Utc>::from_timestamp(1_723_198_514, 0)
            .unwrap()
            .naive_utc()
    }

    fn unique_temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!("zealot-media-{nanos}-{name}"))
    }

    #[test]
    fn file_helpers_match_expected_display_values() {
        let stat = FileStat::new(
            "media/example.JPG",
            Size::from_bytes(1_536),
            false,
            test_timestamp(),
        )
        .unwrap();

        assert_eq!(stat.name(), "example.JPG");
        assert_eq!(stat.extension(), ".jpg");
        assert_eq!(stat.icon(), "🖼️");
        assert_eq!(stat.type_description(), "JPEG image");
        assert_eq!(stat.display_size(), "1.5 KB");
        assert!(stat.is_image());
        assert!(!stat.is_video());
        assert!(!stat.is_audio());
    }

    #[test]
    fn folder_helpers_prefer_folder_specific_values() {
        let stat =
            FileStat::new("media/archive", Size::from_bytes(0), true, test_timestamp()).unwrap();

        assert_eq!(stat.name(), "archive");
        assert_eq!(stat.extension(), "");
        assert_eq!(stat.icon(), "📁");
        assert_eq!(stat.type_description(), "Folder");
        assert!(!stat.is_image());
        assert!(!stat.is_video());
        assert!(!stat.is_audio());
    }

    #[test]
    fn dto_round_trip_preserves_core_fields() {
        let stat = FileStat::new(
            "media/song.mp3",
            Size::from_bytes(42_000),
            false,
            test_timestamp(),
        )
        .unwrap();

        let dto = FileStatDto::from(&stat);
        let round_trip = FileStat::try_from(dto.clone()).unwrap();

        assert_eq!(dto.path, "media/song.mp3");
        assert_eq!(dto.size, 42_000);
        assert_eq!(round_trip.path(), stat.path());
        assert_eq!(round_trip.size_bytes(), stat.size_bytes());
        assert_eq!(round_trip.modified_at(), stat.modified_at());
    }

    #[test]
    fn from_metadata_reads_filesystem_values() {
        let path = unique_temp_path("file.txt");
        fs::write(&path, b"hello").unwrap();

        let metadata = fs::metadata(&path).unwrap();
        let relative_path = PathBuf::from("media/file.txt");
        let stat = FileStat::from_metadata(&relative_path, &metadata).unwrap();

        assert_eq!(stat.path(), relative_path);
        assert_eq!(stat.size_bytes(), 5);
        assert!(!stat.is_folder());
        assert_eq!(stat.extension(), ".txt");

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn new_sanitizes_relative_paths() {
        let stat = FileStat::new(
            "./media//images/./photo.png",
            Size::from_bytes(128),
            false,
            test_timestamp(),
        )
        .unwrap();

        assert_eq!(stat.path(), Path::new("media/images/photo.png"));
        assert_eq!(stat.path_str(), "media/images/photo.png");
    }

    #[test]
    fn new_rejects_parent_dir_paths() {
        let err = FileStat::new(
            "media/../secret.txt",
            Size::from_bytes(128),
            false,
            test_timestamp(),
        )
        .unwrap_err();

        assert!(matches!(err, MediaError::ParentDirNotAllowed { .. }));
    }

    #[test]
    fn new_rejects_absolute_paths() {
        let err = FileStat::new(
            "/media/secret.txt",
            Size::from_bytes(128),
            false,
            test_timestamp(),
        )
        .unwrap_err();

        assert!(matches!(err, MediaError::AbsolutePathNotAllowed { .. }));
    }
}
