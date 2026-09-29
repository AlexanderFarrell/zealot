

export class FileSize {
    private static readonly BytesPerKilobyte = 1024;
    private static readonly BytesPerMegabyte = FileSize.BytesPerKilobyte * 1024;
    private static readonly BytesPerGigabyte = FileSize.BytesPerMegabyte * 1024;
    private static readonly BytesPerTerabyte = FileSize.BytesPerGigabyte * 1024;

    public Bytes: number;

    public constructor(bytes: number) {
        this.Bytes = bytes;
    }

    public static FromBytes(bytes: number): FileSize {
        return new FileSize(bytes);
    }

    public static FromKilobytes(kilobytes: number): FileSize {
        return new FileSize(kilobytes * FileSize.BytesPerKilobyte);
    }

    public static FromMegabytes(megabytes: number): FileSize {
        return new FileSize(megabytes * FileSize.BytesPerMegabyte);
    }

    public static FromGigabytes(gigabytes: number): FileSize {
        return new FileSize(gigabytes * FileSize.BytesPerGigabyte);
    }

    public static FromTerabytes(terabytes: number): FileSize {
        return new FileSize(terabytes * FileSize.BytesPerTerabyte);
    }

    public get Kilobytes(): number {
        return this.Bytes / FileSize.BytesPerKilobyte;
    }

    public get Megabytes(): number {
        return this.Bytes / FileSize.BytesPerMegabyte;
    }

    public get Gigabytes(): number {
        return this.Bytes / FileSize.BytesPerGigabyte;
    }

    public get Terabytes(): number {
        return this.Bytes / FileSize.BytesPerTerabyte;
    }

    public get DisplaySize() {
        if (this.Bytes < FileSize.BytesPerKilobyte) {
            return `${this.Bytes} B`;
        }

        const units = ["KB", "MB", "GB", "TB"];
        let value = this.Kilobytes;
        let i = 0;

        while (value >= FileSize.BytesPerKilobyte && i < units.length - 1) {
            value /= FileSize.BytesPerKilobyte;
            i++;
        }

        const decimals = value >= 10 ? 0 : 1;
        return `${value.toFixed(decimals)} ${units[i]}`;
    }
}

type ActionResult = 
    | {ok: true}
    | {ok: false, message: string};