use tauri::{
    menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu},
    Emitter,
};

const MENU_EVENT: &str = "zealot-menu-command";

fn item<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    id: &str,
    title: &str,
    accelerator: Option<&str>,
) -> tauri::Result<MenuItem<R>> {
    MenuItem::with_id(app, id, title, true, accelerator)
}

fn menu<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<Menu<R>> {
    let new_item = item(app, "new-item", "New Item", Some("CmdOrCtrl+N"))?;
    let new_tab = item(app, "new-tab", "New Tab", Some("CmdOrCtrl+T"))?;
    let close_tab = item(app, "close-tab", "Close Tab", Some("CmdOrCtrl+W"))?;
    let today_note = item(app, "today-note", "Open Today's Note", Some("CmdOrCtrl+Shift+D"))?;
    let random_item = item(app, "random-item", "Open Random Item", None)?;

    let go_back = item(app, "go-back", "Back", Some("CmdOrCtrl+["))?;
    let go_forward = item(app, "go-forward", "Forward", Some("CmdOrCtrl+]"))?;
    let home = item(app, "go-home", "Home", None)?;
    let daily = item(app, "planner-daily", "Today", Some("CmdOrCtrl+1"))?;
    let weekly = item(app, "planner-weekly", "This Week", Some("CmdOrCtrl+2"))?;
    let monthly = item(app, "planner-monthly", "This Month", Some("CmdOrCtrl+3"))?;
    let annual = item(app, "planner-annual", "This Year", Some("CmdOrCtrl+4"))?;
    let time_blocks_day = item(app, "time-blocks-day", "Time Blocks: Today", None)?;
    let time_blocks_week = item(app, "time-blocks-week", "Time Blocks: This Week", None)?;

    let media = item(app, "open-media", "Media", Some("CmdOrCtrl+M"))?;
    let analysis = item(app, "open-analysis", "Analysis", Some("CmdOrCtrl+Shift+1"))?;
    let analysis_recent = item(app, "analysis-recent", "Analysis: Recent", Some("CmdOrCtrl+Shift+R"))?;
    let rules = item(app, "open-rules", "Rules", Some("CmdOrCtrl+Shift+2"))?;
    let types = item(app, "open-types", "Types", Some("CmdOrCtrl+Alt+Shift+T"))?;
    let settings = item(app, "open-settings", "Settings…", Some("CmdOrCtrl+,"))?;

    let search = item(app, "global-search", "Search Zealot", Some("CmdOrCtrl+O"))?;
    let commands = item(app, "command-palette", "Command Palette…", Some("CmdOrCtrl+P"))?;

    let file = Submenu::with_items(
        app,
        "File",
        true,
        &[
            &new_item,
            &new_tab,
            &close_tab,
            &PredefinedMenuItem::separator(app)?,
            &today_note,
            &random_item,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, None)?,
        ],
    )?;
    let edit = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;
    let view = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &go_back,
            &go_forward,
            &home,
            &PredefinedMenuItem::separator(app)?,
            &daily,
            &weekly,
            &monthly,
            &annual,
            &PredefinedMenuItem::separator(app)?,
            &time_blocks_day,
            &time_blocks_week,
        ],
    )?;
    let navigate = Submenu::with_items(
        app,
        "Navigate",
        true,
        &[
            &media,
            &analysis,
            &analysis_recent,
            &rules,
            &types,
            &PredefinedMenuItem::separator(app)?,
            &settings,
        ],
    )?;
    let window = Submenu::with_items(
        app,
        "Window",
        true,
        &[
            &PredefinedMenuItem::minimize(app, None)?,
            &PredefinedMenuItem::fullscreen(app, Some("Enter Full Screen"))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, None)?,
        ],
    )?;
    let help = Submenu::with_items(app, "Help", true, &[&search, &commands])?;

    #[cfg(target_os = "macos")]
    {
        let app_menu = Submenu::with_items(
            app,
            "Zealot",
            true,
            &[
                &PredefinedMenuItem::about(app, Some("About Zealot"), Some(AboutMetadata::default()))?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::services(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::hide(app, None)?,
                &PredefinedMenuItem::hide_others(app, None)?,
                &PredefinedMenuItem::show_all(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::quit(app, None)?,
            ],
        )?;
        Menu::with_items(app, &[&app_menu, &file, &edit, &view, &navigate, &window, &help])
    }

    #[cfg(not(target_os = "macos"))]
    Menu::with_items(app, &[&file, &edit, &view, &navigate, &window, &help])
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .menu(menu)
        .on_menu_event(|app, event| {
            // Native menus own their accelerators, then delegate the actual work to
            // the web UI's existing command runner.
            let _ = app.emit(MENU_EVENT, event.id().0.as_str());
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
