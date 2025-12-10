mod app;
mod macos;

use crate::app::Tile;

use global_hotkey::{
    GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};

fn main() -> iced::Result {
    #[cfg(target_os = "macos")]
    {
        macos::set_activation_policy_regular();
    }

    let manager = GlobalHotKeyManager::new().unwrap();
    let altspace = HotKey::new(Some(Modifiers::ALT), Code::Space);
    //    let esc = HotKey::new(None, Code::Escape);
    manager
        .register_all(&[altspace])
        .expect("Unable to register hotkey");

    iced::daemon(Tile::new, Tile::update, Tile::view)
        .subscription(Tile::subscription)
        .theme(Tile::theme)
        .run()
}
