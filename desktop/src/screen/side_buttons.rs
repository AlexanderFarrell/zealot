use yew::{function_component, html, Html};

#[function_component]
pub fn SideButtons() -> Html {
    html!{
    <div id="side_buttons">
        <button>
            <img src="/public/img/icon/home.svg" title="Home Page" />
        </button>
        <hr />
        <button>
            <img src="/public/img/icon/search.svg" title="Search" />
        </button>
        <button>
            <img src="/public/img/icon/tree2.svg" title="Navigation" />
        </button>
        <button>
            <img src="/public/img/icon/timelapse.svg" title="Calendar" />
        </button>
        <hr />
        <button>
            <img src="/public/img/icon/today.svg" title="Today" />
        </button>
        <button>
            <img src="/public/img/icon/week.svg" title="Week" />
        </button>
        <button>
            <img src="/public/img/icon/items.svg" title="Media" />
        </button>
        <button>
            <img src="/public/img/icon/science.svg" title="Analysis" />
        </button>
        <button>
            <img src="/public/img/icon/settings.svg" title="Settings" />
        </button>
    </div>
    }
}