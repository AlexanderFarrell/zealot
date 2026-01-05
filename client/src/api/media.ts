import { DateTime } from "luxon";
import { delete_req, get_blob, get_json, patch_json, patch_req, post_json, post_req, post_req_form_data } from "../core/api_helper";


// export interface FileStat {
// 	path: string;
// 	size: number;
// 	// image_url: string;
// 	is_folder: boolean;
// }

export const MediaAPI = {
	list_files: async (location: string, page: number = 1): Promise<FileStat[]> => {
		return FileStat.from_array((await get_json(`/api/media/` + location)).files);
	},

	make_folder: async (location: string) => {
		return await post_req(`/api/media/mkdir`, {
			folder: location
		})
	},

	download_file: async (location: string) => {
		const blob = await get_blob(`/api/media/` + location);

		const tempURL = URL.createObjectURL(blob);
		const a = document.createElement("a");
		a.href = tempURL;
		a.download = location.slice(location.lastIndexOf("/") + 1);
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		window.URL.revokeObjectURL(tempURL);
	},

	upload_file: async (file: File, location: string) => {
		const formData = new FormData();
		formData.append("file", file)
		return await post_req_form_data(`/api/media/` + location, formData)
	},
	
	rename: async (old_location: string, new_name: string) => {
		return await patch_req(`/api/media/rename`, {
			old_location,
			new_name
		})
	},

  delete: async (path: string) => {
    return await delete_req(`/api/media/${path}`)
  }
}

export class FileStat {
	path: string;
	size: number;
	is_folder: boolean;
  modified_at: number;

	constructor(path: string, size: number, is_folder: boolean, modified_at: number) {
		this.path = path;
		this.size = size;
		this.is_folder = is_folder;
    this.modified_at = modified_at;
	}

	static from(data: {path: string, size: number, is_folder: boolean, modified_at: number}) {
		return new FileStat(data.path, data.size, data.is_folder, data.modified_at);
	}
	
	static from_array(array: Array<{path: string, size: number, is_folder: boolean, modified_at: number}>) {
		return array.map(item => FileStat.from(item))
	}

	get name(): string {
		const p = this.path.replace(/\/+$/, "")
		const index = p.lastIndexOf("");
		return (index >= 0 && index != p.length) ? p.slice(index + 1) : p;
	}

	get extension(): string {
		if (this.is_folder) return "";
		const name = this.name.toLowerCase();
		const dot_index = name.lastIndexOf(".");
		return dot_index >= 0 ? name.slice(dot_index) : "";
	}

	get icon(): string {
		if (this.is_folder) return "📁";
		return file_icons[this.extension] || "📄";
	}

	get type_description(): string {
		if (this.is_folder) return "Folder";
		return file_type_descriptions[this.extension] || "File";
	}

  get modified_date(): string {
    if (!this.modified_at) return '';
    return DateTime
      .fromSeconds(this.modified_at)
      .toLocaleString(DateTime.DATETIME_MED)
  }

	get is_image(): boolean {
		return this.extension in [
			'.png',
			'.jpg',
			'.jpeg',
			'.gif',
			'.webp',
			'.svg'
		]
	}

	get is_video(): boolean {
		return this.extension in [
			'.mp4',
			'.mov',
			'.mkv',
			'.webm',
			'.avi'
		]
	}

	get is_audio(): boolean {
		return this.extension in [
			'.mp3',
			'.wav',
			'.aac',
			'.m4a',
			'.ogg',
			'.flac'
		]
	}

	get display_size(): string {
		if (this.is_folder) return "";
		const bytes = this.size || 0;
		if (bytes < 1024) return `${bytes} B`;
		const units = ["KB", "MB", "GB", "TB"]
		let value = bytes / 1024;
		let i = 0;
		while (value >= 1024 && i < units.length - 1) {
			value /= 1024;
			i++;
		}
		return `${value.toFixed(1)} ${units[i]}`
	}
}



const file_icons: Record<string, string> = {
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

const file_type_descriptions: Record<string, string> = {
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

export default MediaAPI;
