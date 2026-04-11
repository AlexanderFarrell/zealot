import { BaseElementEmpty, Popups, getNavigator } from '@websoil/engine';
import { MediaAPI } from '@zealot/api/src/media';
import type { FileStat } from '@zealot/domain/src/media';
import { LoadingSpinner } from '../common/loading_spinner';

const mediaApi = new MediaAPI('/api');

export class MediaScreen extends BaseElementEmpty {
    private path: string = '';

    init(path: string): this {
        this.path = path;
        if (this.isConnected) {
            void this.render();
        }
        return this;
    }

    async render() {
        this.className = 'media-screen';
        this.innerHTML = '';
        this.appendChild(new LoadingSpinner());

        let files: FileStat[];
        try {
            files = await mediaApi.ListFiles(this.path);
        } catch (e) {
            this.innerHTML = '';
            Popups.add_error((e as Error).message ?? 'Failed to load media.');
            return;
        }

        this.innerHTML = '';

        this.appendChild(this.buildToolbar());
        this.appendChild(this.buildBreadcrumb());
        this.appendChild(this.buildUploadPanel());
        this.appendChild(this.buildFilesTable(files));
    }

    private buildToolbar(): HTMLElement {
        const toolbar = document.createElement('div');
        toolbar.className = 'media-toolbar';

        const newFolderBtn = document.createElement('button');
        newFolderBtn.type = 'button';
        newFolderBtn.textContent = 'New Folder';
        newFolderBtn.addEventListener('click', () => void this.onNewFolder());

        const uploadBtn = document.createElement('button');
        uploadBtn.type = 'button';
        uploadBtn.textContent = 'Upload';
        uploadBtn.addEventListener('click', () => {
            const panel = this.querySelector<HTMLElement>('.media-upload-panel');
            if (panel) {
                panel.hidden = !panel.hidden;
            }
        });

        toolbar.append(newFolderBtn, uploadBtn);
        return toolbar;
    }

    private buildBreadcrumb(): HTMLElement {
        const nav = document.createElement('nav');
        nav.className = 'media-breadcrumb';

        const root = document.createElement('button');
        root.type = 'button';
        root.textContent = '/';
        root.addEventListener('click', () => getNavigator().openMedia(''));
        nav.appendChild(root);

        const parts = this.path ? this.path.replace(/\/+$/, '').split('/') : [];
        parts.forEach((segment, i) => {
            const sep = document.createElement('span');
            sep.textContent = ' / ';
            nav.appendChild(sep);

            const btn = document.createElement('button');
            btn.type = 'button';
            btn.textContent = segment;
            const segPath = parts.slice(0, i + 1).join('/');
            btn.addEventListener('click', () => getNavigator().openMedia(segPath));
            nav.appendChild(btn);
        });

        return nav;
    }

    private buildUploadPanel(): HTMLElement {
        const panel = document.createElement('div');
        panel.className = 'media-upload-panel';
        panel.hidden = true;

        const fileInput = document.createElement('input');
        fileInput.type = 'file';

        const submitBtn = document.createElement('button');
        submitBtn.type = 'button';
        submitBtn.textContent = 'Upload';
        submitBtn.addEventListener('click', async () => {
            const file = fileInput.files?.[0];
            if (!file) return;
            try {
                await mediaApi.UploadFolder(file, this.path);
                await this.render();
            } catch (e) {
                Popups.add_error((e as Error).message ?? 'Upload failed.');
            }
        });

        panel.append(fileInput, submitBtn);
        return panel;
    }

    private buildFilesTable(files: FileStat[]): HTMLElement {
        const table = document.createElement('table');
        table.className = 'media-table';

        const thead = document.createElement('thead');
        thead.innerHTML = `<tr>
            <th></th>
            <th>Name</th>
            <th>Size</th>
            <th>Type</th>
            <th>Modified</th>
            <th></th>
        </tr>`;
        table.appendChild(thead);

        const tbody = document.createElement('tbody');

        if (files.length === 0) {
            const row = document.createElement('tr');
            const td = document.createElement('td');
            td.colSpan = 6;
            td.textContent = 'No files.';
            td.className = 'media-empty';
            row.appendChild(td);
            tbody.appendChild(row);
        } else {
            files.forEach(file => tbody.appendChild(this.buildFileRow(file)));
        }

        table.appendChild(tbody);
        return table;
    }

    private buildFileRow(file: FileStat): HTMLTableRowElement {
        const row = document.createElement('tr');

        const iconCell = document.createElement('td');
        iconCell.textContent = file.Icon;

        const nameCell = document.createElement('td');
        const nameLink = document.createElement('a');
        nameLink.textContent = file.Name;
        nameLink.href = '#';
        nameLink.addEventListener('click', (e) => {
            e.preventDefault();
            if (file.IsFolder) {
                getNavigator().openMedia(file.Path);
            } else {
                window.open('/api/media/' + file.Path, '_blank');
            }
        });
        nameCell.appendChild(nameLink);

        const sizeCell = document.createElement('td');
        sizeCell.textContent = file.IsFolder ? '—' : file.Size.DisplaySize;

        const typeCell = document.createElement('td');
        typeCell.textContent = file.TypeDescription;

        const modifiedCell = document.createElement('td');
        modifiedCell.textContent = file.ModifiedDateStr;

        const actionsCell = document.createElement('td');
        actionsCell.className = 'media-actions';

        const renameBtn = document.createElement('button');
        renameBtn.type = 'button';
        renameBtn.textContent = 'Rename';
        renameBtn.addEventListener('click', () => void this.onRename(file));

        const deleteBtn = document.createElement('button');
        deleteBtn.type = 'button';
        deleteBtn.textContent = 'Delete';
        deleteBtn.addEventListener('click', () => void this.onDelete(file));

        actionsCell.append(renameBtn, deleteBtn);
        row.append(iconCell, nameCell, sizeCell, typeCell, modifiedCell, actionsCell);
        return row;
    }

    private async onNewFolder(): Promise<void> {
        const name = prompt('Folder name:');
        if (!name || name.trim().length === 0) return;
        const location = this.path ? `${this.path}/${name.trim()}` : name.trim();
        try {
            await mediaApi.MakeFolder(location);
            await this.render();
        } catch (e) {
            Popups.add_error((e as Error).message ?? 'Failed to create folder.');
        }
    }

    private async onRename(file: FileStat): Promise<void> {
        const next = prompt('Rename to:', file.Name);
        if (!next || next === file.Name) return;
        try {
            await mediaApi.Rename(file.Path, next.trim());
            await this.render();
        } catch (e) {
            Popups.add_error((e as Error).message ?? 'Rename failed.');
        }
    }

    private async onDelete(file: FileStat): Promise<void> {
        if (!confirm(`Delete ${file.Name}?`)) return;
        try {
            await mediaApi.Delete(file.Path);
            await this.render();
        } catch (e) {
            Popups.add_error((e as Error).message ?? 'Delete failed.');
        }
    }
}

customElements.define('media-screen', MediaScreen);
