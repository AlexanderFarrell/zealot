use yew::{function_component, html, Html};

use crate::screen::side_buttons::SideButtons;
use crate::screen::side_bar::SideBar;
use crate::screen::content::Content;

#[function_component]
pub fn App() -> Html {
    html! {<>
        <header>
            {"Header"}
        </header>
        <main>
            <SideButtons />
            <SideBar />
            <Content />
            <SideBar />
        </main>
    </>}
}