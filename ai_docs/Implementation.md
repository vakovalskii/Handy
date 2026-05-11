# Краткая реализация по этапам плана (Remote API для Windows + Groq + Custom Models Dir)

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

4. **Произвольная папка для локальных моделей (Завершено)**
   - Добавлена настройка `custom_models_directory: Option<String>` в `AppSettings`.
   - `ModelManager` теперь определяет `models_dir` из настроек при старте; если `custom_models_directory` задана — использует её, иначе — `app_data_dir/models`.
   - `models_dir` обёрнут в `Arc<Mutex<PathBuf>>` для поддержки смены в рантайме.
   - Добавлен метод `set_models_dir()` который переключает папку, пере-сканирует модели и эмитит `models-refreshed`.
   - Добавлены Tauri-команды: `get_models_directory`, `change_custom_models_directory`, `pick_models_directory`.
   - Добавлен UI-компонент `ModelsDirectorySelector` в `AdvancedSettings`.

5. **Корректное переключение между локальными и облачными моделями (Завершено)**
   - При включении `remote_whisper_enabled`:
     - Выгружается текущая локальная модель (`TranscriptionManager::unload_model()`)
     - Очищается `selected_model` в настройках
     - Эмитится событие `settings-changed`
   - При выключении `remote_whisper_enabled`:
     - Если `selected_model` пуст — вызывается `auto_select_model_if_needed()` для выбора первой скачанной модели
     - Запускается предварительная загрузка модели (`initiate_model_load()`)
   - UI (`ModelSelector`, `ModelsSettings`) отображает специальный статус и notice при активном Remote API.
   - `App.tsx` слушает `settings-changed` и обновляет локальное состояние настроек.
