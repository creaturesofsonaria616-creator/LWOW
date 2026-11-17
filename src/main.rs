mod app;
mod store;
mod tabs;
mod themes;
mod webview;

fn main() {
    // Lwow startup sequence:
    // 1. Initialize the data store (SQLite, WAL mode) in the user data dir.
    // 2. Load settings + active theme.
    // 3. Launch the Slint UI shell on the main thread (WebView2 requires COM
    //    init on the UI thread; wry handles this internally).
    // 4. Session recovery happens before the event loop starts, so a crash on
    //    last exit restores tabs without blocking startup.

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
