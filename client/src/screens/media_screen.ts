import API from "../api/api";
import BaseElement from "../components/common/base_element";
import {FileStat} from "../api/media";
import { router } from "../core/router";
import { DeleteIcon, EditIcon, NewFolderIcon, UploadIcon } from "../assets/asset_map";



class MediaScreen extends BaseElement<string> {

    async render() {
        this.classList.add('center');

        this.innerHTML = `
        <h1>Media</h1>
        <div class="media-header row">
            <div class="media-toolbar row">
                <button name="new_folder">
                    <img style="height: 2em" src="${NewFolderIcon}">
                </button>
                <button name="upload">
                    <img style="height: 2em" src="${UploadIcon}">
                </button>
                <button name="rename">
                    <img style="height: 2em" src="${EditIcon}">
                </button>
                <button name="delete">
                    <img style="height: 2em" src="${DeleteIcon}">
                </button>
            </div>
            <breadcrumb-view></breadcrumb-view>
        </div>
        <div name="add" style="display: none;">
            <input type="file">
            <button name="submit">Upload</button>
        </div>
        <div name="dropzone">
            <files-view></files-view>
        </div>
        `

        this.init_header();
        this.init_uploads();
        await this.init_files();


        // files?.forEach((file: FileStat) => {
        //     let file_view = document.createElement('div')
        //     file_view.innerText = file.path;
        //     files_container.appendChild(file_view);
        //     file_view.addEventListener('click', async () => {
        //         if (file.is_folder) {
        //             // Navigate to folder
        //             router.navigate('/media/' + file.path);
        //         } else {
        //             await API.media.download_file(this.data! + "/" + file.path)
        //         }
        //     })
        // })
        // if (files == null) {
        //     files_container.innerHTML = "Empty"
        // }


    }

    private init_header() {
        const breadcrumb = this.querySelector('breadcrumb-view')! as BreadcrumbView;
        breadcrumb.init(this.data!);

        const new_folder_button = this.querySelector('[name="new_folder"]')! as HTMLButtonElement;
        const rename_button = this.querySelector('[name="rename"]')! as HTMLButtonElement;
        const delete_button = this.querySelector('[name="delete"]')! as HTMLButtonElement;

        new_folder_button.addEventListener('click', async () => {
            const name = prompt("Folder name: ")
            if (!name || name.length == 0) return;
            const path = this.data + "/" + name;
            await API.media.make_folder(path);
            await this.init_files();
        })

        rename_button.addEventListener('click', async () => {
            const files_container = this.querySelector('files-view')! as FilesView;
            let selected = files_container.selected;
            if (!selected) return;
            const next = prompt("Rename to:", selected.name);
            if (!next || next == selected.name) return;

            const old_path = this.data + "/" + selected.path;

            // Rename
            await API.media.rename(old_path, next);

            files_container.clear_selected();
            await this.init_files();
        })

        delete_button.addEventListener('click', async () => {
            const files_container = this.querySelector('files-view')! as FilesView;
            let selected = files_container.selected;
            if (!selected) return;

            const confirm_delete = confirm(`Delete ${selected.name}?`);
            if (!confirm_delete) return;

            const path = this.data + "/" + selected.path;
            await API.media.delete(path)

            files_container.clear_selected();
            await this.init_files();
        })
    }

    private init_uploads() {
        const upload_button = this.querySelector('[name="upload"]')! as HTMLButtonElement;
        const upload_view = this.querySelector('[name="add"]')! as HTMLDivElement;
        let enabled = false;
        upload_button.addEventListener('click', () => {
            enabled = !enabled;
            if (enabled) {
                upload_view.style.display = 'block'
            } else {
                upload_view.style.display = 'none';
            }
        })


        let upload_input = this.querySelector('[type="file"]')! as HTMLInputElement;
        let upload_submit = this.querySelector('[name="submit"]')! as HTMLButtonElement;
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

    private async init_files() {
        const files_container = this.querySelector('files-view')! as FilesView;
        const dropzone = this.querySelector('[name="dropzone"]')! as HTMLDivElement;

        const files = await API.media.list_files(this.data!);

        files_container.init(files)
    }
}

type FileViewType = 'table' | 'icons';

class FilesView extends BaseElement<FileStat[]> {
    private _kind: FileViewType = 'table';
    private _selected: FileStat | null = null;

    public set kind(value: FileViewType) {
        this._kind = value;
        this.render();
    }

    public get selected(): FileStat | null {
        return this._selected;
    }

    public clear_selected() {
        this._selected = null;
        this.querySelectorAll('.selected').forEach(r => r.classList.remove('selected'))
    }

    render() {
        if (this.data == null) {
            this.innerText = "No files.";
            return;
        }

        if (this._kind == "table") {
            this.render_table();
        }
        else if (this._kind == "icons") {
            this.render_icons();
        }
        else {
            this.innerText = "Invalid File View Type";
        }
    }

    private render_table() {
        const files = this.data!;
        this.innerHTML = `<table>
            <thead>
                <td>Icon</td>
                <td>Name</td>
                <td>Size</td>
                <td>Type</td>
                <td>Modified</td>
            </thead>
            <tbody>
            </tbody>
        </table>`

        const body = this.querySelector('tbody')!;

        files.forEach(file => {
            let row = document.createElement('tr')
            // Get extension

            row.innerHTML = `
            <td>${file.icon}</td>
            <td><a>${file.name}</a></td>
            <td>${file.display_size}</td>
            <td>${file.type_description}</td>
            <td>${file.modified_date}</td>
            `
            row.querySelector('a')?.addEventListener('click', (event: Event) => {
                event.preventDefault();
                event.stopPropagation();
                if (file.is_folder) {
                    router.navigate('/media/' + file.path);
                } else {
                    API.media.download_file(file.path)
                }
            })
            row.addEventListener('click', () => {
                body.querySelectorAll('tr').forEach(r => r.classList.remove('selected'))
                row.classList.add('selected')
                this._selected = file;
            })
            body?.appendChild(row)
        })
    }

    private render_icons() {
        const files = this.data!;

    }


}

// class FileView extends BaseElement<FileStat> {
//     render() {

//     }
// }

class BreadcrumbView extends BaseElement<string> {
    private get currentPath(): string {
        return (this.data || "").replace(/^\/+|\/+$/g, "");
    }

    render() {
        const parts = this.currentPath ? this.currentPath.split("/") : [];
        this.classList.add('row');

        // Root
        let root = document.createElement('button');
        root.innerText = "/";
        root.style.fontSize = "2em";
        root.addEventListener('click', () => {
            router.navigate('/media');
        })
        this.appendChild(root);

        // Each part
        for (let i = 0; i < parts.length; i++) {
            const stem = parts[i];
            const path = parts.slice(0, i).join('/');
            
            const button = document.createElement('button');
            button.innerText = stem;
            button.style.fontSize = "2em";
            button.addEventListener('click', () => {
                router.navigate('/media/' + path)
            })
            this.appendChild(button)
        }
    }
}




customElements.define('media-screen', MediaScreen);
customElements.define('breadcrumb-view', BreadcrumbView);
customElements.define('files-view', FilesView);

export default MediaScreen;
