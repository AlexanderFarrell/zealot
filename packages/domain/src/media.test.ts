import { describe, expect, it } from "vitest";
import { FileStat, FileIcons, FileTypeDescriptions, type FileStatDto } from "./media";

function stat(path: string, overrides: Partial<FileStatDto> = {}): FileStat {
    return new FileStat({ path, size: 2048, is_folder: false, modified_at: 1_700_000_000, ...overrides });
}

describe("FileStat", () => {
    it("maps dto fields", () => {
        const file = stat("notes/a.md");
        expect(file.Path).toBe("notes/a.md");
        expect(file.Size.Kilobytes).toBe(2);
        expect(file.IsFolder).toBe(false);
        expect(file.ModifiedAt.toSeconds()).toBe(1_700_000_000);
        expect(file.ModifiedDateStr).not.toBe("");
    });

    it("takes the last path segment as the name and ignores trailing slashes", () => {
        expect(stat("a/b/c.txt").Name).toBe("c.txt");
        expect(stat("a/b/").Name).toBe("b");
        expect(stat("plain.txt").Name).toBe("plain.txt");
    });

    it("lowercases the extension and handles names without one", () => {
        expect(stat("a/Photo.JPG").Extension).toBe(".jpg");
        expect(stat("a/Makefile").Extension).toBe("");
        expect(stat("a/archive.tar.gz").Extension).toBe(".gz");
    });

    it("gives folders no extension, a folder icon and description", () => {
        const folder = stat("a/dir.d", { is_folder: true });
        expect(folder.Extension).toBe("");
        expect(folder.Icon).toBe("📁");
        expect(folder.TypeDescription).toBe("Folder");
    });

    it("looks up icon and description by extension with fallbacks", () => {
        expect(stat("a.png").Icon).toBe(FileIcons[".png"]);
        expect(stat("a.png").TypeDescription).toBe(FileTypeDescriptions[".png"]);
        expect(stat("a.unknown").Icon).toBe("📄");
        expect(stat("a.unknown").TypeDescription).toBe("File");
    });

    it("classifies media kinds by extension", () => {
        expect(stat("a.png").IsImage).toBe(true);
        expect(stat("a.mp4").IsVideo).toBe(true);
        expect(stat("a.flac").IsAudio).toBe(true);
        expect(stat("a.txt").IsImage).toBe(false);
        expect(stat("a.png").IsVideo).toBe(false);
        expect(stat("a.mp3").IsImage).toBe(false);
        expect(stat("noext").IsAudio).toBe(false);
    });

    it("has an icon and a description for every known extension", () => {
        expect(Object.keys(FileIcons).sort()).toEqual(Object.keys(FileTypeDescriptions).sort());
    });
});
