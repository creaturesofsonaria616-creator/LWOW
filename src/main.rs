mod app;
mod store;
mod tabs;
mod themes;
mod webviews;

fn main() {
    // Lwow startup sequence:
    // 1. Resolve and create the user data directory (SQLite lives there).
    // 2. Open the store (WAL mode) and load settings + active theme.
    // 3. Hand everything to app::run, which instantiates the Slint shell,
    //    restores the last session, and attaches WebView2 children per tab.

    let data_dir = dirs::data_local_dir()
        .expect("could not resolve local data directory")
        .join("Lwow");
    std::fs::create_dir_all(&data_dir).expect("could not create Lwow data directory");

    let store = store::Store::open(&data_dir).expect("could not open Lwow database");
    let theme = themes::Theme::load_active(&store);
    let settings = app::Settings::load(&store);

    let app_state = app::AppState::new(store, theme, settings);

    app::run(app_state);
}
