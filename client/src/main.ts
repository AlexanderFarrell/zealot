import './assets/style.css'
import typescriptLogo from './typescript.svg'
import viteLogo from '/vite.svg'
import './components/side_buttons.ts';
import './components/titlebar.ts';
import './components/content.ts';
// import {}
// import { setupCounter } from './counter.ts'

document.querySelector<HTMLDivElement>('#app')!.innerHTML = 
`<title-bar></title-bar>
<side-buttons></side-buttons>
<content></content>`

