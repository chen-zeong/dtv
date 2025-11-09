use std::sync::Mutex;

#[cfg(target_os = "macos")]
use core_foundation::string::CFStringRef;

#[derive(Default)]
pub struct KeepAwakeManager {
    counter: Mutex<u32>,
    #[cfg(target_os = "macos")]
    assertion_id: Mutex<Option<u32>>,
    #[cfg(target_os = "windows")]
    windows_active: Mutex<bool>,
}

impl KeepAwakeManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn begin(&self) {
        let mut counter = self.counter.lock().unwrap();
        *counter += 1;
        if *counter == 1 {
            self.activate_platform();
        }
    }

    pub fn end(&self) {
        let mut counter = self.counter.lock().unwrap();
        if *counter == 0 {
            return;
        }
        *counter -= 1;
        if *counter == 0 {
            self.deactivate_platform();
        }
    }

    #[cfg(target_os = "macos")]
    fn activate_platform(&self) {
        use core_foundation::base::TCFType;
        use core_foundation::string::CFString;

        const ASSERTION_LEVEL_ON: u32 = 255; // kIOPMAssertionLevelOn

        let assertion_type = CFString::new("PreventUserIdleSystemSleep");
        let assertion_name = CFString::new("DTV Live Playback");

        let mut assertion_id: u32 = 0;
        let result = unsafe {
            IOPMAssertionCreateWithName(
                assertion_type.as_concrete_TypeRef(),
                ASSERTION_LEVEL_ON,
                assertion_name.as_concrete_TypeRef(),
                &mut assertion_id as *mut u32,
            )
        };

        if result == KERN_SUCCESS {
            *self.assertion_id.lock().unwrap() = Some(assertion_id);
        } else {
            eprintln!(
                "[KeepAwake] Failed to create macOS power assertion, IOReturn={}",
                result
            );
        }
    }

    #[cfg(target_os = "macos")]
    fn deactivate_platform(&self) {
        if let Some(id) = self.assertion_id.lock().unwrap().take() {
            let result = unsafe { IOPMAssertionRelease(id) };
            if result != KERN_SUCCESS {
                eprintln!(
                    "[KeepAwake] Failed to release macOS power assertion, IOReturn={}",
                    result
                );
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn activate_platform(&self) {
        use windows_sys::Win32::System::Power::{
            SetThreadExecutionState, ES_AWAYMODE_REQUIRED, ES_CONTINUOUS, ES_SYSTEM_REQUIRED,
        };

        let result = unsafe {
            SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_AWAYMODE_REQUIRED)
        };
        if result != 0 {
            *self.windows_active.lock().unwrap() = true;
        } else {
            eprintln!("[KeepAwake] SetThreadExecutionState failed to enable keep-awake");
        }
    }

    #[cfg(target_os = "windows")]
    fn deactivate_platform(&self) {
        use windows_sys::Win32::System::Power::{SetThreadExecutionState, ES_CONTINUOUS};

        if !*self.windows_active.lock().unwrap() {
            return;
        }
        let result = unsafe { SetThreadExecutionState(ES_CONTINUOUS) };
        if result == 0 {
            eprintln!("[KeepAwake] SetThreadExecutionState failed to reset state");
        }
        *self.windows_active.lock().unwrap() = false;
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn activate_platform(&self) {}

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn deactivate_platform(&self) {}
}

#[cfg(target_os = "macos")]
type IOReturn = i32;

#[cfg(target_os = "macos")]
const KERN_SUCCESS: IOReturn = 0;

#[cfg(target_os = "macos")]
#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOPMAssertionCreateWithName(
        assertionType: CFStringRef,
        assertionLevel: u32,
        assertionName: CFStringRef,
        assertionID: *mut u32,
    ) -> IOReturn;

    fn IOPMAssertionRelease(assertionID: u32) -> IOReturn;
}
