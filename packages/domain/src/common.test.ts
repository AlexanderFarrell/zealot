import { describe, expect, it } from "vitest";
import { FileSize } from "./common";

describe("FileSize", () => {
    it("converts between units", () => {
        expect(FileSize.FromKilobytes(1).Bytes).toBe(1024);
        expect(FileSize.FromMegabytes(2).Bytes).toBe(2 * 1024 ** 2);
        expect(FileSize.FromGigabytes(1).Bytes).toBe(1024 ** 3);
        expect(FileSize.FromTerabytes(1).Bytes).toBe(1024 ** 4);
        expect(FileSize.FromBytes(2048).Kilobytes).toBe(2);
        expect(FileSize.FromGigabytes(3).Megabytes).toBe(3072);
        expect(FileSize.FromTerabytes(1).Gigabytes).toBe(1024);
        expect(FileSize.FromGigabytes(1).Terabytes).toBeCloseTo(1 / 1024);
    });

    it.each([
        [0, "0 B"],
        [1023, "1023 B"],
        [1024, "1.0 KB"],
        [1536, "1.5 KB"],
        [10 * 1024, "10 KB"],
        [5 * 1024 ** 2, "5.0 MB"],
        [3 * 1024 ** 3, "3.0 GB"],
        [2 * 1024 ** 4, "2.0 TB"],
        [5000 * 1024 ** 4, "5000 TB"],
    ])("displays %d bytes as %s", (bytes, expected) => {
        expect(new FileSize(bytes).DisplaySize).toBe(expected);
    });
});
