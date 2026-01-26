//! This is the haptics module for rustcast
//! It is used to perform haptic feedback on the macOS platform
//! For debugging this module, just pray and hope that it works, because even I don't fully understand
//! how it works
#![allow(non_camel_case_types)]

use objc2_core_foundation::{CFNumber, CFNumberType, CFRetained, CFString, CFType};
use std::{
    ffi::{c_char, c_void},
    sync::LazyLock,
};

use crate::platform::HapticPattern;

unsafe extern "C" {
    unsafe fn CFRelease(cf: *mut CFType);
}

/// Convert a pattern to an index
#[inline]
fn pattern_index(pattern: HapticPattern) -> i32 {
    match pattern {
        HapticPattern::Generic => 0,
        HapticPattern::Alignment => 1,
        HapticPattern::LevelChange => 2,
    }
}

type kern_return_t = i32;
type io_object_t = u32;
type io_iterator_t = u32;
type io_registry_entry_t = u32;
type mach_port_t = u32;

unsafe extern "C" {
    fn IOServiceMatching(name: *const c_char) -> *mut CFType;
    fn IOServiceGetMatchingServices(
        master: mach_port_t,
        matching: *mut CFType,
        iter: *mut io_iterator_t,
    ) -> kern_return_t;
    fn IOIteratorNext(iter: io_iterator_t) -> io_object_t;
    fn IOObjectRelease(obj: io_object_t) -> kern_return_t;
    fn IORegistryEntryCreateCFProperty(
        entry: io_registry_entry_t,
        key: *mut CFString,
        allocator: *const c_void,
        options: u32,
    ) -> *mut CFType;

    fn MTActuatorCreateFromDeviceID(device_id: u64) -> *mut CFType;
    fn MTActuatorOpen(actuator: *mut CFType) -> i32; // IOReturn
    fn MTActuatorIsOpen(actuator: *mut CFType) -> bool;
    fn MTActuatorActuate(actuator: *mut CFType, pattern: i32, unk: i32, f1: f32, f2: f32) -> i32;

    fn CFGetTypeID(cf: *mut CFType) -> usize;
    fn CFNumberGetTypeID() -> usize;
    fn CFNumberGetValue(number: *mut CFNumber, theType: i32, valuePtr: *mut u64) -> bool;
}

#[inline]
fn k_iomain_port_default() -> mach_port_t {
    0
}

struct MtsState {
    actuators: Vec<*mut CFType>,
}

unsafe impl Send for MtsState {}
unsafe impl Sync for MtsState {}

impl MtsState {
    fn open_default_or_all() -> Option<Self> {
        let mut iter: io_iterator_t = 0;
        unsafe {
            let name = c"AppleMultitouchDevice";
            let matching = IOServiceMatching(name.as_ptr());
            if matching.is_null() {
                return None;
            }
            if IOServiceGetMatchingServices(k_iomain_port_default(), matching, &mut iter) != 0 {
                return None;
            }
        }

        let key = CFString::from_str("Multitouch ID");
        let mut actuators: Vec<*mut CFType> = Vec::new();

        unsafe {
            loop {
                let dev = IOIteratorNext(iter);
                if dev == 0 {
                    break;
                }

                let id_ref = IORegistryEntryCreateCFProperty(
                    dev,
                    CFRetained::<CFString>::as_ptr(&key).as_ptr(),
                    std::ptr::null(),
                    0,
                );

                if !id_ref.is_null() && CFGetTypeID(id_ref) == CFNumberGetTypeID() {
                    let mut device_id: u64 = 0;
                    if CFNumberGetValue(
                        id_ref as *mut CFNumber,
                        CFNumberType::SInt64Type.0 as i32,
                        &mut device_id as *mut u64,
                    ) {
                        let act = MTActuatorCreateFromDeviceID(device_id);
                        if !act.is_null() {
                            if MTActuatorOpen(act) == 0 {
                                actuators.push(act);
                            } else {
                                CFRelease(act);
                            }
                        }
                    }
                }

                if !id_ref.is_null() {
                    CFRelease(id_ref);
                }
                IOObjectRelease(dev);
            }

            if iter != 0 {
                IOObjectRelease(iter);
            }
        }

        if actuators.is_empty() {
            None
        } else {
            Some(Self { actuators })
        }
    }
}

static MTS: LazyLock<Option<MtsState>> = LazyLock::new(MtsState::open_default_or_all);

fn mts_state() -> Option<&'static MtsState> {
    MTS.as_ref()
}

/// Perform a haptic feedback - Just use this function to perform haptic feedback... please don't
/// remake this function unless you're a genius or absolutely have to
pub(crate) fn perform_haptic(pattern: HapticPattern) -> bool {
    let Some(state) = mts_state() else {
        return false;
    };

    let pat = pattern_index(pattern);
    let mut any_ok = false;

    unsafe {
        for &act in &state.actuators {
            if !act.is_null() && MTActuatorIsOpen(act) {
                let kr = MTActuatorActuate(act, pat, 0, 0.0, 0.0);
                any_ok |= kr == 0;
            }
        }
    }

    any_ok
}
