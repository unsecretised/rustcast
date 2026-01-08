//! This has the menubar icon logic for the app
use objc2::{MainThreadMarker, sel};
use objc2_app_kit::{NSImage, NSMenu, NSMenuItem, NSStatusBar, NSVariableStatusItemLength};
use objc2_foundation::{NSSize, NSString};

/// This create a new menubar icon for the app
pub fn new_menu_icon(mtm: MainThreadMarker) {
    let status_bar = NSStatusBar::systemStatusBar();
    let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);

    if let Some(button) = status_item.button(mtm) {
        if let Some(image) = NSImage::imageNamed(&NSString::from_str("icon")) {
            image.setSize(NSSize {
                width: 25.,
                height: 25.,
            });
            button.setImage(Some(&image));
        } else {
            button.setTitle(&NSString::from_str("RustCast"));
        }
    }

    let menu = NSMenu::new(mtm);
    menu.setAutoenablesItems(false);

    let quit_title = NSString::from_str("Quit RustCast");
    let quit_item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            &quit_title,
            sel!(terminate:).into(),
            &NSString::from_str("q"),
        )
    };

    let version_title = NSString::from_str(
        &("RustCast Version: ".to_string()
            + option_env!("APP_VERSION").unwrap_or("Unknown Version")),
    );
    let version_item = NSMenuItem::new(mtm);
    version_item.setTitle(&version_title);
    version_item.setEnabled(false);

    menu.addItem(&quit_item);
    menu.addItem(&version_item);

    status_item.setMenu(Some(&menu));
    status_item.setVisible(true);
}
