# Рабочий лог

**2026-05-05**
- Изучена кодовая база `d:\Project\Handy_VaKovalskii`.
- Обнаружено, что функционал Remote Whisper API (для подключения удаленных моделей распознавания аудио) уже кроссплатформенный и работает на Windows через `reqwest`.
- Создан системный план `implementation_plan.md` с запросом уточнений у пользователя.
- Созданы файлы `Implementation.md` и `last_state.md` в папке `ai_docs` (ранее я случайно проверял папку `ai_docs` из другого проекта в метаданных).
- Ожидаю уточнений от пользователя по поводу того, какой именно функционал API имелся в виду.
- [ОБНОВЛЕНИЕ] Пользователь пояснил, что нужно интегрировать провайдер Groq, а также задал вопрос про служебные клавиши в качестве хоткеев.
- Изучена документация Groq. Выяснено, что API Groq совместимо с OpenAI, поэтому бэкенд на Rust (с использованием `reqwest` и `multipart`) уже подходит для этого.
- В план реализации добавлено создание селектора провайдеров в UI (`RemoteWhisperSettings.tsx`), где при выборе "Groq" будет подставляться нужный URL и модели.
- По поводу хоткеев (Правый Ctrl/Alt): стандартная реализация Tauri (`tauri-plugin-global-shortcut`) не позволяет одиночные модификаторы. Сторонняя библиотека `handy-keys` разрешает, но обычно не различает левый и правый. Предложено написать нативный Windows-хук `WH_KEYBOARD_LL` (подобно прошлому проекту пользователя).
- Обновлен `implementation_plan.md` с открытыми вопросами к пользователю, ожидается подтверждение.
- [ЗАВЕРШЕНИЕ ИНТЕГРАЦИИ] Пользователь подтвердил план, и выполнение переведено в новую ветку `feature/remote-groq-native-hotkey`. 
- **Groq API**: Добавлен `Provider` селектор в `RemoteWhisperSettings.tsx`. Настроена автоматическая подстановка `https://api.groq.com/openai/v1` и нужной модели. 
- **Native Hotkeys**: Написан нативный хук `WH_KEYBOARD_LL` в `windows_native.rs`. Обновлен `Cargo.toml` (`Win32_UI_Input_KeyboardAndMouse`). Добавлен игнор одиночных модификаторов в `tauri_impl.rs` для Windows. Модифицированы `keyboard.ts` и `GlobalShortcutInput.tsx` для сохранения точного указания модификатора (например, `Right Control`) в UI.
- Задачи выполнены, запускается процесс верификации (компиляция Rust-кода через `cargo check`).

**2026-05-08**
- **Сборка на Windows**: Установлен CMake через `winget`. Исправлены bundled bindings `whisper-rs-sys` (layout-тесты для Linux-типов, тип `whisper_gretype`). Установлен `WHISPER_DONT_GENERATE_BINDINGS=1` в `.cargo/config.toml`. Адаптирован `windows_native.rs` под `windows` крейт v0.61.3 (перемещены `KBDLLHOOKSTRUCT`/`WH_KEYBOARD_LL`, `None` вместо конструкторов handle, `Option<HHOOK>` для `CallNextHookEx`). Сборка завершилась успешно.

**2026-05-10**
- **Onboarding: обход для Remote API**: Обнаружено, что `checkOnboardingStatus` в `App.tsx` проверяет только наличие скачанных локальных моделей. Пользователи, желающие использовать Groq API, застревали на экране выбора модели.
  - `App.tsx`: добавлена проверка `remote_whisper_enabled` — если включён, onboarding модели пропускается.
  - `Onboarding.tsx`: добавлена кнопка «Или используйте Remote API (Groq, OpenAI) →» внизу экрана. При нажатии включается `remote_whisper_enabled` и переход к основному интерфейсу.
  - Добавлен импорт `useSettingsStore` в `Onboarding.tsx`.
