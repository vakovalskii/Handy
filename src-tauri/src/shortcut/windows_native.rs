#[cfg(target_os = "windows")]
pub mod hook {
    use log::{error, info};
    use std::ptr::null_mut;
    use std::sync::Once;
    use std::thread;
    use tauri::AppHandle;
    use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
        HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN,
        WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    };

    use crate::settings;
    use crate::shortcut::handler::handle_shortcut_event;

    static mut APP_HANDLE: Option<AppHandle> = None;
    static mut HOOK_HANDLE: HHOOK = HHOOK(std::ptr::null_mut());
    static INIT_ONCE: Once = Once::new();

    pub fn init(app: AppHandle) {
        INIT_ONCE.call_once(|| {
            unsafe {
                APP_HANDLE = Some(app);
            }

            thread::spawn(|| {
                unsafe {
                    let hook = SetWindowsHookExW(
                        WH_KEYBOARD_LL,
                        Some(hook_callback),
                        None,
                        0,
                    );

                    if hook.is_err() {
                        error!("Failed to install WH_KEYBOARD_LL hook");
                        return;
                    }

                    HOOK_HANDLE = hook.unwrap();
                    info!("Windows native keyboard hook installed successfully");

                    let mut msg = MSG::default();
                    while GetMessageW(&mut msg, None, 0, 0).into() {
                        // Standard message loop to keep the hook alive
                    }

                    let _ = UnhookWindowsHookEx(HOOK_HANDLE);
                }
            });
        });
    }

    const LLKHF_EXTENDED: u32 = 0x01;

    unsafe extern "system" fn hook_callback(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if ncode >= 0 {
            let kb_struct = *(lparam.0 as *const KBDLLHOOKSTRUCT);
            let vk_code = kb_struct.vkCode;
            let is_extended = (kb_struct.flags.0 & LLKHF_EXTENDED) != 0;

            let key_name = match vk_code {
                162 => "ControlLeft",
                163 => "ControlRight",
                164 => "AltLeft",
                165 => "AltRight",
                160 => "ShiftLeft",
                161 => "ShiftRight",
                17 => {
                    if is_extended {
                        "ControlRight"
                    } else {
                        "ControlLeft"
                    }
                }
                18 => {
                    if is_extended {
                        "AltRight"
                    } else {
                        "AltLeft"
                    }
                }
                16 => {
                    if is_extended {
                        "ShiftRight"
                    } else {
                        "ShiftLeft"
                    }
                }
                _ => "",
            };

            if !key_name.is_empty() {
                let is_pressed =
                    wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN;
                let is_released = wparam.0 as u32 == WM_KEYUP || wparam.0 as u32 == WM_SYSKEYUP;

                if is_pressed || is_released {
                    if let Some(app) = APP_HANDLE.as_ref() {
                        let settings = settings::get_settings(app);
                        
                        let mut matched_binding_id = None;
                        let key_name_lower = key_name.to_lowercase();
                        
                        for (id, binding) in &settings.bindings {
                            let bound_key = binding.current_binding.to_lowercase();
                            if bound_key == key_name_lower {
                                matched_binding_id = Some(id.clone());
                                break;
                            }
                            if key_name_lower == "controlright" && bound_key == "rightcontrol" {
                                matched_binding_id = Some(id.clone());
                                break;
                            }
                            if key_name_lower == "altright" && bound_key == "rightalt" {
                                matched_binding_id = Some(id.clone());
                                break;
                            }
                            if key_name_lower == "controlleft" && bound_key == "leftcontrol" {
                                matched_binding_id = Some(id.clone());
                                break;
                            }
                            if key_name_lower == "altleft" && bound_key == "leftalt" {
                                matched_binding_id = Some(id.clone());
                                break;
                            }
                            if key_name_lower == "shiftright" && bound_key == "rightshift" {
                                matched_binding_id = Some(id.clone());
                                break;
                            }
                            if key_name_lower == "shiftleft" && bound_key == "leftshift" {
                                matched_binding_id = Some(id.clone());
                                break;
                            }
                        }

                        if let Some(binding_id) = matched_binding_id {
                            let app_clone = app.clone();
                            let binding_id_clone = binding_id.clone();
                            let hotkey_string = key_name.to_string();
                            
                            tauri::async_runtime::spawn(async move {
                                handle_shortcut_event(
                                    &app_clone,
                                    &binding_id_clone,
                                    &hotkey_string,
                                    is_pressed,
                                );
                            });
                        }
                    }
                }
            }
        }

        CallNextHookEx(Some(HOOK_HANDLE), ncode, wparam, lparam)
    }
}
