import { get_blob, get_json, post_json, post_req_form_data } from "../core/api_helper";


export interface FileStat {
	path: string;
	size: number;
	// image_url: string;
	is_folder: boolean;
}

export const MediaAPI = {
	list_files: async (location: string, page: number = 1): Promise<FileStat[]> => {
		return (await get_json(`/api/media/` + location)).files;
	},

	make_folder: async (location: string) => {
		return await post_json(`/api/media/mkdir`, {
			folder: location
		})
	},

	download_file: async (location: string) => {
		const blob = await get_blob(`/api/media/` + location);

		const tempURL = URL.createObjectURL(blob);
		const a = document.createElement("a");
		a.href = tempURL;
		a.download = location.slice(location.lastIndexOf("/") + 1);
		a.click();

		document.body.removeChild(a);
		window.URL.revokeObjectURL(tempURL);
	},

	upload_file: async (file: File, location: string) => {
		const formData = new FormData();
		formData.append("file", file)
		return await post_req_form_data(`/api/media/` + location, formData)
	}
}

export default MediaAPI;