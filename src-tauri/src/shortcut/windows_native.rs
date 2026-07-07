#[cfg(target_os = "windows")]
pub mod hook {
    use log::{error, info};
    use std::collections::HashSet;
    use std::sync::{LazyLock, Mutex, Once, OnceLock};
    use std::thread;
    use tauri::AppHandle;
    use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG,
        WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    };

    use crate::settings;
    use crate::shortcut::handler::handle_shortcut_event;

    static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
    static INIT_ONCE: Once = Once::new();
    static PRESSED_KEYS: LazyLock<Mutex<HashSet<&'static str>>> =
        LazyLock::new(|| Mutex::new(HashSet::new()));

    fn should_emit_key_event_for_set(
        pressed_keys: &mut HashSet<&'static str>,
        key_name: &'static str,
        is_pressed: bool,
        is_released: bool,
    ) -> bool {
        if is_pressed {
            return pressed_keys.insert(key_name);
        }

        if is_released {
            return pressed_keys.remove(key_name);
        }

        false
    }

    fn should_emit_key_event(key_name: &'static str, is_pressed: bool, is_released: bool) -> bool {
        let Ok(mut pressed_keys) = PRESSED_KEYS.lock() else {
            return false;
        };

        should_emit_key_event_for_set(&mut pressed_keys, key_name, is_pressed, is_released)
    }

    pub fn init(app: AppHandle) {
        INIT_ONCE.call_once(|| {
            let _ = APP_HANDLE.set(app);

            thread::spawn(|| {
                unsafe {
                    let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_callback), None, 0);

                    let Ok(hook) = hook else {
                        error!("Failed to install WH_KEYBOARD_LL hook");
                        return;
                    };

                    info!("Windows native keyboard hook installed successfully");

                    let mut msg = MSG::default();
                    while GetMessageW(&mut msg, None, 0, 0).into() {
                        // Standard message loop to keep the hook alive
                    }

                    let _ = UnhookWindowsHookEx(hook);
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
                let is_pressed = wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN;
                let is_released = wparam.0 as u32 == WM_KEYUP || wparam.0 as u32 == WM_SYSKEYUP;

                if (is_pressed || is_released)
                    && should_emit_key_event(key_name, is_pressed, is_released)
                {
                    if let Some(app) = APP_HANDLE.get() {
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

        CallNextHookEx(None, ncode, wparam, lparam)
    }

    #[cfg(test)]
    mod tests {
        use super::should_emit_key_event_for_set;
        use std::collections::HashSet;

        #[test]
        fn suppresses_repeated_modifier_keydown_until_release() {
            let mut pressed_keys = HashSet::new();

            assert!(should_emit_key_event_for_set(
                &mut pressed_keys,
                "ControlRight",
                true,
                false
            ));
            assert!(!should_emit_key_event_for_set(
                &mut pressed_keys,
                "ControlRight",
                true,
                false
            ));
            assert!(should_emit_key_event_for_set(
                &mut pressed_keys,
                "ControlRight",
                false,
                true
            ));
        }

        #[test]
        fn suppresses_release_without_prior_press() {
            let mut pressed_keys = HashSet::new();

            assert!(!should_emit_key_event_for_set(
                &mut pressed_keys,
                "ControlRight",
                false,
                true
            ));
        }
    }
}
