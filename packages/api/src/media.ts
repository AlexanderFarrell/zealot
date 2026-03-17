import { FileStat, FileStatDto } from "@zealot/domain/src/media";
import { BaseAPI } from "./common";
import { delete_req, get_json, patch_req, post_req, post_req_form_data } from "@websoil/engine";


export class MediaAPI extends BaseAPI {
    public constructor(baseURL: string) {
        super(baseURL)
    }

    public async ListFiles(location: string, page: number = 1): Promise<FileStat[]> {
        let json = await get_json(`${this.baseUrl}/media/${location}?page=${page}`);
        let dtos: FileStatDto[] = json['files'];
        return dtos.map(d => new FileStat(d));
    }

    public async MakeFolder(location: string) {
        return await post_req(`${this.baseUrl}/media/mkdir`, {
            folder: location
        })
    }

    public async UploadFolder(file: File, location: string) {
        const formData = new FormData();
        formData.append('file', file);
        return await post_req_form_data(`${this.baseUrl}/media/${location}`, formData);
    }

    public async Rename(old_location: string, new_name: string) {
        return await patch_req(`${this.baseUrl}/media/rename`, {
            old_location,
            new_name
        })
    }

    public async Delete(path: string) {
        return await delete_req(`${this.baseUrl}/media/${path}`)
    }
}