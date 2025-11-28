class TypeSettingsScreen extends HTMLElement {
    connectedCallback() {
        this.innerHTML = `
        <h1>Type Settings</h1>
        <h2>Types</h2>
        <form id='add_type'></form>
        <div id='types_container'></div>
        
        <h2>Attribute Kinds</h2>
        <form id='add_attribute_kind'></form>
        <div id='attribute_kinds_container'></div>
        `;

        (this.querySelector('#add_type') as HTMLFormElement)!.addEventListener('submit', (e: SubmitEvent) => {
                        
        });

        (this.querySelector('#add_attribute_kind') as HTMLFormElement)!.addEventListener('submit', (e: SubmitEvent) => {

        });

        this.refresh();
    }

    refresh() {
        let types_container = this.querySelector("#types_container")!;
        let attribute_kinds_container = this.querySelector('#attribute_kinds_container')!;
    }
    

    disconnectedCallback() {

    }
}

customElements.define('type-settings-screen', TypeSettingsScreen);

export default TypeSettingsScreen;