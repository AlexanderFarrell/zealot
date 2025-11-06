import './style.css'
import typescriptLogo from './typescript.svg'
import viteLogo from '/vite.svg'
import titlebar from './components/titlebar.ts';
// import {}
// import { setupCounter } from './counter.ts'

document.querySelector<HTMLDivElement>('#app')!.innerHTML = 
`<title-bar></title-bar>
<side_buttons></side_buttons>
<content></content>`

