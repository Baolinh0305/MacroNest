#[cfg(windows)]
mod windows_platform {
    use std::env;

    use anyhow::{Result, bail};
    use windows::{
        Win32::{
            Foundation::{CloseHandle, GetLastError, HANDLE, HWND},
            System::Threading::{
                CreateMutexW, GetCurrentProcess, HIGH_PRIORITY_CLASS, SetPriorityClass,
            },
            UI::{
                Shell::{IsUserAnAdmin, ShellExecuteW},
                WindowsAndMessaging::{MB_ICONWARNING, MB_OK, MessageBoxW, SW_SHOWNORMAL},
            },
        },
        core::{PCWSTR, w},
    };

    const MUTEX_NAME: &str = "Global\\CrosshairOverlaySingleInstance";

    pub struct SingleInstanceGuard {
        handle: HANDLE,
    }

    impl Drop for SingleInstanceGuard {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.handle);
            }
        }
    }

    pub fn relaunch_as_admin_if_needed() -> Result<bool> {
        unsafe {
            if IsUserAnAdmin().as_bool() {
                return Ok(false);
            }
        }

        let exe = env::current_exe()?;
        let exe_wide = widestring(exe.as_os_str().to_string_lossy().as_ref());
        unsafe {
            let result = ShellExecuteW(
                Some(HWND(std::ptr::null_mut())),
                w!("runas"),
                PCWSTR(exe_wide.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );
            if (result.0 as usize) <= 32 {
                bail!("Administrator elevation was cancelled or failed");
            }
        }
        Ok(true)
    }

    pub fn acquire_single_instance() -> Result<Option<SingleInstanceGuard>> {
        let name = widestring(MUTEX_NAME);
        let handle = unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr()))? };
        let already_exists = unsafe { GetLastError().0 } == windows::Win32::Foundation::ERROR_ALREADY_EXISTS.0;
        if already_exists {
            unsafe {
                let _ = CloseHandle(handle);
                let _ = MessageBoxW(
                    Some(HWND(std::ptr::null_mut())),
                    w!("MacroNest is already running. Please close the existing tray icon before launching another instance."),
                    w!("MacroNest"),
                    MB_OK | MB_ICONWARNING,
                );
            }
            return Ok(None);
        }

        Ok(Some(SingleInstanceGuard { handle }))
    }

    pub fn set_high_priority() {
        unsafe {
            let _ = SetPriorityClass(GetCurrentProcess(), HIGH_PRIORITY_CLASS);
        }
    }

    fn widestring(value: &str) -> Vec<u16> {
        let mut wide: Vec<u16> = value.encode_utf16().collect();
        wide.push(0);
        wide
    }
}

#[cfg(windows)]
pub use windows_platform::*;

#[cfg(not(windows))]
mod fallback {
    use anyhow::Result;

    pub struct SingleInstanceGuard;

    pub fn relaunch_as_admin_if_needed() -> Result<bool> {
        Ok(false)
    }

    pub fn acquire_single_instance() -> Result<Option<SingleInstanceGuard>> {
        Ok(Some(SingleInstanceGuard))
    }

    pub fn set_high_priority() {}
}

#[cfg(not(windows))]
pub use fallback::*;
