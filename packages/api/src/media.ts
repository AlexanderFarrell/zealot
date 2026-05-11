import { FileStat, type FileStatDto } from "@zealot/domain/src/media";
import { BaseAPI } from "./common";
import { delete_req, get_json, patch_req, post_req, post_req_form_data } from "@websoil/engine";


export class MediaAPI extends BaseAPI {
    public constructor(baseURL: string) {
        super(baseURL)
    }

    public async ListFiles(location: string, page: number = 1): Promise<FileStat[]> {
        const loc = location.replace(/\/+$/, '');
        const url = loc ? `${this.baseUrl}/media/${loc}?page=${page}` : `${this.baseUrl}/media?page=${page}`;
        let json = await get_json(url);
        let dtos: FileStatDto[] = json['files'];
        return dtos.map(d => new FileStat(d));
    }

    public async MakeFolder(location: string) {
        return await post_req(`${this.baseUrl}/media/mkdir`, {
            folder: location
        })
    }

    public async UploadFolder(file: File, location: string) {
        const loc = location.replace(/\/+$/, '');
        const url = loc ? `${this.baseUrl}/media/${loc}` : `${this.baseUrl}/media`;
        const formData = new FormData();
        formData.append('file', file);
        return await post_req_form_data(url, formData);
    }

    public async Rename(old_location: string, new_name: string) {
        return await patch_req(`${this.baseUrl}/media/rename`, {
            old_location,
            new_name
        })
    }

    public async Delete(path: string) {
        return await delete_req(`${this.baseUrl}/media/${path.replace(/\/+$/, '')}`)
    }
}