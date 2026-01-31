//! File with the functions to *statically* get an icon, bundled into the binary

// Smol macros for DRY purposes
macro_rules! static_geticon {
    ($name:ident, $bytes:expr, $sz:literal) => {
        #[allow(unused)]
        pub fn $name() -> iced::window::Icon {
            let icon = image::load_from_memory($bytes).unwrap();

            iced::window::icon::from_rgba(icon.as_bytes().to_vec(), $sz, $sz).unwrap()
        }
    };
}

macro_rules! static_geticon_imghandle {
    ($name:ident, $bytes:expr) => {
        #[allow(unused)]
        pub fn $name() -> iced::widget::image::Handle {
            iced::widget::image::Handle::from_bytes($bytes)
        }
    };
}

// const IMG_64: &[u8] = include_bytes!("../assets/icon/icon64.png");
// const IMG_128: &[u8] = include_bytes!("../assets/icon/icon128.png");
const IMG_256: &[u8] = include_bytes!("../assets/icon/icon256.png");
// const IMG_512: &[u8] = include_bytes!("../assets/icon/icon512.png");

pub mod iced_icon {
    use super::*;

    // static_geticon!(icon_64, IMG_64, 64);
    // static_geticon!(icon_128, IMG_128, 128);
    static_geticon!(icon_256, IMG_256, 256);
    // static_geticon!(icon_512, IMG_512, 512);
}

pub mod iced_img_handle {
    use super::*;

    // static_geticon_imghandle!(icon_64, IMG_64);
    // static_geticon_imghandle!(icon_128, IMG_128);
    static_geticon_imghandle!(icon_256, IMG_256);
    // static_geticon_imghandle!(icon_512, IMG_512);
}
