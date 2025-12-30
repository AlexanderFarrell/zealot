import API from "../api/api";
import BaseElement from "../components/common/base_element";
import type { FileStat } from "../api/media";

class MediaScreen extends BaseElement<string> {
    async render() {
        let files = await API.media.list_files(this.data!);

        this.innerHTML = `
        <h1>Media</h1>
        <div>${this.data!}</div>
        <div name="add">
            <input type="file">
            <button name="submit">Upload</button>
        </div>
        <div name="files"></div>
        `

        let files_container = this.querySelector('[name="files"]')! as HTMLDivElement;
        let upload_input = this.querySelector('[type="file"]')! as HTMLInputElement;
        let upload_submit = this.querySelector('[name="submit"]')! as HTMLButtonElement;

        files?.forEach((file: FileStat) => {
            let file_view = document.createElement('div')
            file_view.innerText = file.path;
            files_container.appendChild(file_view);
            file_view.addEventListener('click', async () => {
                await API.media.download_file(this.data! + "/" + file.path)
            })
        })
        if (files == null) {
            files_container.innerHTML = "Empty"
        }

        upload_submit.addEventListener('click', async () => {
            const file = upload_input.files?.[0];
            if (!file) {
                return;
            }

            await API.media.upload_file(file, this.data!);
            upload_input.value = "";
            this.render();
        })
    }
}

customElements.define('media-screen', MediaScreen)

export default MediaScreen;
