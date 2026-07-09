#[cfg(target_os = "windows")]
pub mod hook {
    use log::{debug, error, info};
    use std::collections::HashMap;
    use std::sync::{LazyLock, Mutex, Once, OnceLock};
    use std::thread;
    use std::time::{Duration, Instant};
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
    static PRESSED_KEYS: LazyLock<Mutex<HashMap<&'static str, Instant>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));
    const STALE_KEYDOWN_TIMEOUT: Duration = Duration::from_secs(2);

    fn should_emit_key_event_for_map(
        pressed_keys: &mut HashMap<&'static str, Instant>,
        key_name: &'static str,
        is_pressed: bool,
        is_released: bool,
        now: Instant,
    ) -> bool {
        if is_pressed {
            if let Some(previous_press) = pressed_keys.get_mut(key_name) {
                if now.duration_since(*previous_press) > STALE_KEYDOWN_TIMEOUT {
                    *previous_press = now;
                    return true;
                }

                return false;
            }

            pressed_keys.insert(key_name, now);
            return true;
        }

        if is_released {
            return pressed_keys.remove(key_name).is_some();
        }

        false
    }

    fn should_emit_key_event(key_name: &'static str, is_pressed: bool, is_released: bool) -> bool {
        let Ok(mut pressed_keys) = PRESSED_KEYS.lock() else {
            return false;
        };

        should_emit_key_event_for_map(
            &mut pressed_keys,
            key_name,
            is_pressed,
            is_released,
            Instant::now(),
        )
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
                            debug!(
                                "Windows native shortcut matched: binding_id={}, key={}, pressed={}",
                                binding_id, key_name, is_pressed
                            );

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
        use super::{should_emit_key_event_for_map, STALE_KEYDOWN_TIMEOUT};
        use std::collections::HashMap;
        use std::time::Instant;

        #[test]
        fn suppresses_repeated_modifier_keydown_until_release() {
            let mut pressed_keys = HashMap::new();
            let now = Instant::now();

            assert!(should_emit_key_event_for_map(
                &mut pressed_keys,
                "ControlRight",
                true,
                false,
                now
            ));
            assert!(!should_emit_key_event_for_map(
                &mut pressed_keys,
                "ControlRight",
                true,
                false,
                now + STALE_KEYDOWN_TIMEOUT / 2
            ));
            assert!(should_emit_key_event_for_map(
                &mut pressed_keys,
                "ControlRight",
                false,
                true,
                now + STALE_KEYDOWN_TIMEOUT / 2
            ));
        }

        #[test]
        fn suppresses_release_without_prior_press() {
            let mut pressed_keys = HashMap::new();

            assert!(!should_emit_key_event_for_map(
                &mut pressed_keys,
                "ControlRight",
                false,
                true,
                Instant::now()
            ));
        }

        #[test]
        fn allows_new_keydown_after_stale_pressed_state() {
            let mut pressed_keys = HashMap::new();
            let first_press = Instant::now();
            let second_press = first_press + STALE_KEYDOWN_TIMEOUT + STALE_KEYDOWN_TIMEOUT;

            assert!(should_emit_key_event_for_map(
                &mut pressed_keys,
                "ControlRight",
                true,
                false,
                first_press
            ));
            assert!(should_emit_key_event_for_map(
                &mut pressed_keys,
                "ControlRight",
                true,
                false,
                second_press
            ));
        }
    }
}
