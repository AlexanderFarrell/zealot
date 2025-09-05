use yew::{function_component, html, Html};

#[function_component]
pub fn SideButtons() -> Html {
    html!{
    <div id="side_buttons">
        <button>{"A"}</button>
        <button>{"B"}</button>
        <button>{"C"}</button>
    </div>
    }
}