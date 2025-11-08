import IconButton from './common/icon_button.ts';
import HomeIcon from '../assets/icon/home.svg';
import NavIcon from '../assets/icon/tree2.svg';
import SearchIcon from '../assets/icon/search.svg';

class SideButtons extends HTMLElement {

    connectedCallback() {
        this.appendChild(
            new IconButton(
                HomeIcon,
                "Go to Home Page",
                () => {console.log("Home Page")}
            )
        )
        this.appendChild(
            new IconButton(
                SearchIcon,
                "Search Items",
                () => {console.log("Search Items")}
            )
        )
        this.appendChild(
            new IconButton(
                NavIcon,
                "Navigation View",
                () => {console.log("Navigation View")}
            )
        )
    }
}

customElements.define('side-buttons', SideButtons)

export default SideButtons;