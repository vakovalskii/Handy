# Краткая реализация по этапам плана (Remote API для Windows + Groq)

1. **Анализ текущего состояния проекта**
   - Установлено, что функционал Remote Whisper API (`remote_whisper.rs`) работает кроссплатформенно с использованием `reqwest`.
   
2. **Интеграция Groq API (Завершено)**
   - Добавлен UI-пресет `Provider` в `RemoteWhisperSettings.tsx` с вариантами: `Custom`, `OpenAI`, `Groq`.
   - При выборе `Groq` автоматически подставляются URL (`https://api.groq.com/openai/v1`) и модель `whisper-large-v3-turbo`. Поле URL блокируется для изменения, чтобы предотвратить опечатки.

3. **Служебные клавиши в качестве Hotkey (Завершено)**
   - Реализован нативный `WH_KEYBOARD_LL` хук для Windows (в `src-tauri/src/shortcut/windows_native.rs`).
   - Хук надежно перехватывает `RightControl`, `RightAlt`, `LeftControl`, `LeftAlt` и другие левые/правые модификаторы независимо от языковой раскладки.
   - Изменен `tauri_impl.rs` для игнорирования одиночных модификаторов при проверке Tauri-шорткатов.
   - Обновлен фронтенд `GlobalShortcutInput.tsx` и `keyboard.ts`, чтобы они сохраняли и отображали точные названия левых/правых модификаторов (например `Right Control`), но конвертировали их в базовые (например `Ctrl`) в составе сложных комбинаций.
