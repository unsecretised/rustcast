//! Extracts icons from executables etc.

use std::path::Path;

use iced::widget;
use widestring::U16CString;
use windows::{
    Win32::{
        Foundation::TRUE,
        Graphics::Gdi::{
            BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC,
            DeleteObject, GetDIBits, HBITMAP, SelectObject,
        },
        UI::{
            Shell::ExtractIconExW,
            WindowsAndMessaging::{DestroyIcon, GetIconInfoExW, HICON, ICONINFOEXW},
        },
    },
    core::PCWSTR,
};

use crate::utils::bgra_to_rgba;

/// Gets the icons from an executable.
///
/// Adapted from an answer to https://stackoverflow.com/questions/7819024
///
/// # Errors
///
/// - If the path contains a NUL byte before the end
/// - Any internal win32 error
pub fn get_first_icon(path: impl AsRef<Path>) -> anyhow::Result<Option<widget::image::Handle>> {
    let path = path.as_ref();

    let path_cstr = U16CString::from_os_str(path.as_os_str())?;
    let path_pcwstr = PCWSTR(path_cstr.as_ptr());

    let icon_count = unsafe { ExtractIconExW(path_pcwstr, -1, None, None, 0) };

    // Don't bother doing the rest
    if icon_count == 0 {
        return Ok(None);
    }

    let mut large_icons = vec![HICON::default(); icon_count as usize];
    let mut small_icons = vec![HICON::default(); icon_count as usize];

    let icons_fetched = unsafe {
        ExtractIconExW(
            path_pcwstr,
            0,
            Some(large_icons.as_mut_ptr()),
            Some(small_icons.as_mut_ptr()),
            icon_count,
        )
    };

    tracing::trace!(
        target: "icon_fetch",
        "{icons_fetched}/{icon_count} icons fetched for {}",
        path.display()
    );

    let hicon = large_icons.iter().chain(small_icons.iter()).next();

    if let Some(hicon) = hicon {
        let res = hicon_to_imghandle(*hicon);
        unsafe { DestroyIcon(*hicon) }?;
        Ok(Some(res?)) // Error only gets propogated down here, so that hicon is always destroyed
    } else {
        Ok(None)
    }
}

fn hicon_to_imghandle(hicon: HICON) -> Result<widget::image::Handle, windows::core::Error> {
    let mut icon_info = ICONINFOEXW {
        cbSize: size_of::<ICONINFOEXW>() as u32,
        fIcon: TRUE,
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: HBITMAP::default(),
        hbmColor: HBITMAP::default(),
        wResID: 0,
        szModName: unsafe { std::mem::zeroed() },
        szResName: unsafe { std::mem::zeroed() },
    };

    let result = unsafe { GetIconInfoExW(hicon, &mut icon_info) };

    // Nonzero return values indicate ok, while zero means error
    if result.0 == 0 {
        return Err(windows::core::Error::from_win32());
    }

    let (bitmap_info, bitmap) = get_icon_bitmap(icon_info)?;

    let BITMAPINFOHEADER {
        biWidth, biHeight, ..
    } = bitmap_info.bmiHeader;

    debug_assert_eq!(biWidth * -biHeight * 4, bitmap.len() as i32);
    let data = widget::image::Handle::from_rgba(biWidth as u32, (-biHeight) as u32, bitmap);

    Ok(data)
}

fn get_icon_bitmap(icon_info: ICONINFOEXW) -> Result<(BITMAPINFO, Vec<u8>), windows::core::Error> {
    let hdc_screen = unsafe { CreateCompatibleDC(None) };
    let hdc_mem = unsafe { CreateCompatibleDC(hdc_screen) };
    let hbm_old = unsafe { SelectObject(hdc_mem, icon_info.hbmColor) };

    let mut bmp_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: icon_info.xHotspot as i32 * 2,
            biHeight: -(icon_info.yHotspot as i32 * 2),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: DIB_RGB_COLORS.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut buffer: Vec<u8> =
        vec![0; (icon_info.xHotspot * 2 * icon_info.yHotspot * 2 * 4) as usize];

    let gdib_result = unsafe {
        GetDIBits(
            hdc_mem,
            icon_info.hbmColor,
            0,
            icon_info.yHotspot * 2,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmp_info,
            DIB_RGB_COLORS,
        )
    };

    // It's just stored here because it should still go through to the cleanup code
    let val = if gdib_result == 0 {
        Err(windows::core::Error::from_win32())
    } else {
        bgra_to_rgba(buffer.as_mut_slice());
        Ok((bmp_info, buffer))
    };

    // cleanup
    unsafe {
        SelectObject(hdc_mem, hbm_old);
        DeleteDC(hdc_mem).ok()?;
        DeleteDC(hdc_screen).ok()?;
        DeleteObject(icon_info.hbmColor).ok()?;
        DeleteObject(icon_info.hbmMask).ok()?;
    }

    val
}
