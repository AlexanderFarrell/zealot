import { DateTime } from "luxon";
import { FileSize } from "./common";


export class FileStat {
    public Path: string;
    public Size: FileSize;
    public IsFolder: boolean;
    public ModifiedAt: DateTime;

    public constructor(dto: FileStatDto) {
        this.Path = dto.path;
        this.Size = new FileSize(dto.size);
        this.IsFolder = dto.is_folder;
        this.ModifiedAt = DateTime .fromSeconds(dto.modified_at);
    }

    public get Name(): string {
		const p = this.Path.replace(/\/+$/, "")
		const index = p.lastIndexOf("");
		return (index >= 0 && index != p.length) ? p.slice(index + 1) : p;
    }

    public get Extension(): string {
        if (this.IsFolder) return "";
		const name = this.Name.toLowerCase();
		const dot_index = name.lastIndexOf(".");
		return dot_index >= 0 ? name.slice(dot_index) : "";
    }

    public get Icon(): string {
		if (this.IsFolder) return "📁";
		return FileIcons[this.Extension] || "📄";
    }

    public get TypeDescription(): string {
        if (this.IsFolder) return "Folder";
        return FileTypeDescriptions[this.Extension] || "File";
    }

    public get ModifiedDateStr(): string {
        return this.ModifiedAt.toLocaleString(DateTime.DATETIME_MED);
    }
}

export interface FileStatDto {
    path: string;
    size: number;
    is_folder: boolean;
    modified_at: number;
}

export const FileIcons: Record<string, string> = {
  // Text / docs
  ".txt": "📄",
  ".md": "📄",
  ".rtf": "📄",
  ".pdf": "📄",
  ".doc": "📄",
  ".docx": "📄",
  ".odt": "📄",

  // Spreadsheets
  ".csv": "📊",
  ".tsv": "📊",
  ".xls": "📊",
  ".xlsx": "📊",
  ".ods": "📊",

  // Slides
  ".ppt": "📊",
  ".pptx": "📊",
  ".odp": "📊",

  // Images
  ".png": "🖼️",
  ".jpg": "🖼️",
  ".jpeg": "🖼️",
  ".gif": "🖼️",
  ".bmp": "🖼️",
  ".webp": "🖼️",
  ".tiff": "🖼️",
  ".svg": "🖼️",
  ".ico": "🖼️",
  ".heic": "🖼️",
  ".avif": "🖼️",

  // Video
  ".mp4": "🎬",
  ".mov": "🎬",
  ".mkv": "🎬",
  ".webm": "🎬",
  ".avi": "🎬",
  ".m4v": "🎬",
  ".wmv": "🎬",

  // Audio
  ".mp3": "🎵",
  ".wav": "🎵",
  ".aac": "🎵",
  ".m4a": "🎵",
  ".flac": "🎵",
  ".ogg": "🎵",
  ".opus": "🎵",

  // Archives
  ".zip": "🗜️",
  ".rar": "🗜️",
  ".7z": "🗜️",
  ".tar": "🗜️",
  ".gz": "🗜️",
  ".bz2": "🗜️",
  ".xz": "🗜️",

  // Books
  ".epub": "📕",
  ".mobi": "📕",

  // Code
  ".js": "💻",
  ".ts": "💻",
  ".jsx": "💻",
  ".tsx": "💻",
  ".json": "💻",
  ".html": "💻",
  ".css": "💻",
  ".scss": "💻",
  ".go": "💻",
  ".py": "💻",
  ".rb": "💻",
  ".php": "💻",
  ".java": "💻",
  ".c": "💻",
  ".cpp": "💻",
  ".rs": "💻",
  ".sh": "💻",
  ".sql": "💻",
  ".yml": "💻",
  ".yaml": "💻",

  // Design
  ".psd": "🎨",
  ".ai": "🎨",
  ".fig": "🎨",
  ".sketch": "🎨",
};

export const FileTypeDescriptions: Record<string, string> = {
  // Text / docs
  ".txt": "Text file",
  ".md": "Markdown document",
  ".rtf": "Rich Text document",
  ".pdf": "PDF document",
  ".doc": "Word document",
  ".docx": "Word document",
  ".odt": "OpenDocument text",

  // Spreadsheets
  ".csv": "Comma Separated Values file",
  ".tsv": "Tab Separated Values file",
  ".xls": "Excel spreadsheet",
  ".xlsx": "Excel spreadsheet",
  ".ods": "OpenDocument spreadsheet",

  // Slides
  ".ppt": "PowerPoint presentation",
  ".pptx": "PowerPoint presentation",
  ".odp": "OpenDocument presentation",

  // Images
  ".png": "PNG image",
  ".jpg": "JPEG image",
  ".jpeg": "JPEG image",
  ".gif": "GIF image",
  ".bmp": "Bitmap image",
  ".webp": "WebP image",
  ".tiff": "TIFF image",
  ".svg": "SVG image",
  ".ico": "Icon image",
  ".heic": "HEIC image",
  ".avif": "AVIF image",

  // Video
  ".mp4": "MP4 video",
  ".mov": "QuickTime video",
  ".mkv": "Matroska video",
  ".webm": "WebM video",
  ".avi": "AVI video",
  ".m4v": "M4V video",
  ".wmv": "WMV video",

  // Audio
  ".mp3": "MP3 audio",
  ".wav": "WAV audio",
  ".aac": "AAC audio",
  ".m4a": "M4A audio",
  ".flac": "FLAC audio",
  ".ogg": "Ogg audio",
  ".opus": "Opus audio",

  // Archives
  ".zip": "ZIP archive",
  ".rar": "RAR archive",
  ".7z": "7-Zip archive",
  ".tar": "TAR archive",
  ".gz": "Gzip archive",
  ".bz2": "Bzip2 archive",
  ".xz": "XZ archive",

  // Books
  ".epub": "EPUB book",
  ".mobi": "MOBI book",

  // Code
  ".js": "JavaScript file",
  ".ts": "TypeScript file",
  ".jsx": "JSX file",
  ".tsx": "TSX file",
  ".json": "JSON file",
  ".html": "HTML document",
  ".css": "CSS stylesheet",
  ".scss": "SCSS stylesheet",
  ".go": "Go source file",
  ".py": "Python file",
  ".rb": "Ruby file",
  ".php": "PHP file",
  ".java": "Java source file",
  ".c": "C source file",
  ".cpp": "C++ source file",
  ".rs": "Rust source file",
  ".sh": "Shell script",
  ".sql": "SQL file",
  ".yml": "YAML file",
  ".yaml": "YAML file",

  // Design
  ".psd": "Photoshop document",
  ".ai": "Illustrator document",
  ".fig": "Figma design",
  ".sketch": "Sketch design",
};
