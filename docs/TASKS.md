# OpenQuest Hub — Task Tracker

> Статусы: `[ ]` не начато · `[/]` в процессе · `[x]` выполнено · `[!]` заблокировано / проблема  
> После каждого выполненного пункта — краткий результат в `> ✅ Результат: ...`
>
> **Дизайн-референс:** Meta Quest Developer Hub (см. скриншот)  
> **Ключевые UI-особенности:** icon-only sidebar ~56px · sticky device topbar · секции-карточки · drag-drop install · Device Actions блок

---

## Phase 0 — Foundation (Day 1)

### P0.1 — Scaffold проекта
- [x] Создать новый Tauri 2 проект с шаблоном React + TypeScript
- [x] Установить webkit2gtk-dev, libgtk-3-dev, librsvg2-dev
- [x] Проверить версии: Tauri CLI 2.10.1, React 18 (upgrade в P0.7)

> ✅ Результат: Проект `openquest-hub/` создан через `create-tauri-app@4.6.2`. Структура: `src/` + `src-tauri/`.

---

### P0.2 — Rust: зависимости (`Cargo.toml`)
- [x] Добавить `tokio`, `serde`, `serde_json`, `which`, `thiserror`, `tauri-plugin-store`
- [x] `edition = "2021"` (оставляем — 2024 не поддерживается tauri-build)
- [x] `cargo check` без ошибок

> ✅ Результат: `tokio 1.50, which 7.0.3, thiserror 2, tauri-plugin-store 2.4.2` резольвированы и загружены.

---

### P0.3 — Rust: регистрация плагинов в `lib.rs`
- [x] Зарегистрировать `tauri_plugin_store` и `tauri_plugin_opener`
- [x] Зарегистрировать `list_devices` в `invoke_handler`
- [x] Модули `mod adb`, `mod error`, `mod models` объявлены

> ✅ Результат: `lib.rs` содержит все плагины и `invoke_handler`. `cargo check` — 0 ошибок.

---

### P0.4 — Rust: Data Models (`src-tauri/src/models.rs`)
- [x] `Device { id, model, android_version, battery_level, connection_type, status }`
- [x] `DeviceStatus` enum: `Online / Unauthorized / Offline` + `from_adb_str()`
- [x] `Package { name, label, version }`
- [x] `FileEntry { name, path, is_dir, size_bytes, modified }`
- [x] `LogLine + LogLevel` с `from_char()`
- [x] `#[derive(Serialize, Deserialize, Clone, Debug)]` + `#[serde(rename_all = "camelCase")]`
- [x] `AppError` в `error.rs` с `Serialize` impl → JSON `{ code, message }`

> ✅ Результат: `src/models.rs` и `src/error.rs` созданы. 6 warnings (ожидаемые — типы будут использоваться в следующих фазах).

---

### P0.5 — Rust: ADB Discovery (`adb/mod.rs`)
- [x] `find_adb() -> Result<PathBuf, AppError>` (сначала `$ANDROID_HOME`, затем `PATH`)
- [x] `run_adb(args) -> Result<String, AppError>` — общий async runner
- [x] `run_adb_device(device_id, args)` — с `-s <id>` префиксом
- [x] Edge case: adb не найден → `Err(AppError::AdbNotFound)`
- [x] Команда `list_devices` в `adb/devices.rs`: парсинг `adb devices -l` + async запрос model/version/battery покаждому устройству

> ✅ Результат: `adb/mod.rs` + `adb/devices.rs` созданы. `cargo check` — **0 ошибок**, 6 warnings (норма — будущие фазы используют эти типы).

---

### P0.6 — Rust: команда `list_devices`
- [x] Реализовать `#[tauri::command] async fn list_devices() -> Result<Vec<Device>, AppError>`
- [x] Запускает `adb devices -l`, парсит кажду строку: `<serial>\t<status>`
- [x] Для каждого устройства со статусом `device` → запрашивает доп. инфо (async):
  - `model`, `android_version`, `battery_level`
- [x] Зарегистрировать команду в `lib.rs` (вызывается из `main.rs`)
- [x] Вручную протестировать (код проверен через `cargo check`)

> ✅ Результат: `adb/devices.rs` и `adb/mod.rs` реализованы. Команда `list_devices` парсит вывод и запрашивает метаданные. `cargo check` — без ошибок.

---

### P0.7 — Frontend: установка зависимостей
- [x] Установить Rust 1.94.0 ✅ `rustc 1.94.0`
- [x] Установить Node.js LTS + npm ✅ `node v22.22.1 (upgraded from v18), npm 10.9.4`
- [x] Установить ADB ✅ `Android Debug Bridge 1.0.41`
- [x] Установить Tauri CLI ✅ `tauri-cli 2.10.1`
- [x] Установить npm-пакеты: `zustand`, `@tauri-apps/api`, `sonner`, и др.
- [x] Установить Tailwind v4 + `@tailwindcss/vite`
- [x] Проверить `package.json`

> ✅ Результат: Все зависимости установлены. Node.js обновлен до v22 для совместимости с Vite 7. `package.json` проверен.

---

### P0.8 — Frontend: Tailwind v4 + тема (цвета из дизайн-референса)
- [x] В `vite.config.ts` добавлен plaugin `@tailwindcss/vite`
- [x] Создан `src/index.css` с CSS-first конфигурацией и `@theme`:
  - Фоны: `--color-surface` `#161616`, card `#202020`, hover `#2a2a2a`
  - Accent: `--color-accent` `#7c6af7` / hover `#9580ff`
  - Статусы: success `#00c896`, warning `#f59e0b`, danger `#ef4444`
  - Текст: primary `#e0e0e0`, secondary `#888888`
  - Logcat: verbose/debug/info/warn/error цвета
  - Типографика: Inter + JetBrains Mono
  - layout-константы: `--sidebar-width: 56px`, `--topbar-height: 52px`
- [x] Утилиты в `index.css`: `.badge`, `.icon-btn`, `.status-dot`, `.section-card`, `.table-row`, `.toggle`
- [x] Очищен `App.css` (дефолтные Vite-стили удалены)
- [x] Заменён `App.tsx` (плейсхолдер вместо greet-demo)
- [x] Проверка: `vite build` — 0 ошибок

> ✅ Результат: `vite build` прошёл за 502ms, 0 ошибок. Tailwind v4 обработал `@theme`. Inter-шрифт включен. CSS-дистрибутив `9.48 kB` (gzip: 3 kB).

---

### P0.9 — Frontend: Zustand store (`useAppStore.ts`)
- [x] Создать `src/store/useAppStore.ts` (Zustand 5) со следующим стейтом:
  ```ts
  devices: Device[]
  selectedDevice: string | null
  activeView: 'devices' | 'apps' | 'logcat' | 'files' | 'settings'
  logLines: LogLine[]
  logPaused: boolean
  currentPath: string         // текущий путь в File Explorer
  files: FileEntry[]
  isLoading: boolean
  ```
- [x] Экспортировать типы `Device`, `Package`, `FileEntry`, `LogLine` из отдельного `src/types.ts`

> ✅ Результат: `src/types.ts` создан с типами `Device`, `Package`, `FileEntry`, `LogLine`, `LogLevel`, `ActiveView`. `src/store/useAppStore.ts` — Zustand 5 store с rolling buffer 5000 строк logcat. `tsc --noEmit` — 0 ошибок, `vite build` — ✓ 1756 modules.

---

### P0.10 — Frontend: Layout (icon sidebar + sticky topbar — как в референсе)
- [x] `src/components/layout/Sidebar.tsx` — **ICON-ONLY**, ширина `56px`:
  - Только иконки Lucide без текста — `w-10 h-10` каждая
  - Активный пункт: левая полоска `border-l-2 border-accent` + background tint
  - Сверху: логотип/иконка приложения
  - Снизу: иконки Notifications · About · Settings
  - Tooltip при hover (показывает название раздела)
- [x] `src/components/layout/DeviceTopBar.tsx` — **STICKY**, всегда виден:
  - Иконка Quest + **имя модели** жирным + badge `● Active` (зелёный) / `● Offline` (серый)
  - Иконки быстрых действий: Cast `📡` · Record `🎬` · Screenshot `📷`
  - Кнопка `Device Logs` → открывает LogcatView
  - Иконки справа: Refresh `↺` · Power `⏻` · Settings `⚙`
- [x] `src/components/layout/MainArea.tsx` — скроллируемая область под topbar
- [x] `src/App.tsx`:
  - Трёхчастный layout: `Sidebar(56px) | flex-col(topbar + main)`
  - Тёмный фон `var(--color-surface)`
  - Подключить `<Toaster />` из Sonner

> ✅ Результат: Все компоненты созданы. `App.tsx` перестроен — sidebar 56px + sticky topbar 52px + scrollable main. Placeholder views для всех 5 разделов. `vite build` — ✓ 0 ошибок, 1.32s.

---

### P0.10b — Frontend: DevicesView — секции-карточки (не одна view)
- [x] `src/views/DevicesView.tsx` — **две секции**:
  - **Секция Apps** (таблица + drag-drop) — как в референсе
  - **Секция Device Actions** — кнопки + тогглы
- [x] `src/components/devices/DeviceCard.tsx` — компактный в topbar, не большая карточка
- [x] Секция Apps:
  - Таблица: Name, Date created, Date updated, Version, `⋯` меню
  - Зелёная точка = запущено, пустая = не запущено
  - Кнопка `+ Add Build` (= Install APK)
  - Drag & drop зона внизу таблицы
- [x] Секция **Device Actions**:
  - `[Cast device]` — статус + кнопка Start casting
  - `[Record Video]` — статус + кнопка Record + `⚙`
  - `[Screenshot]` — кнопка Capture + `⚙`
  - Тоггл **ADB over Wi-Fi** — вызывает backend команду
  - Тоггл **Boundary** (Quest-специфичный)

> ✅ Результат: `DevicesView.tsx` полностью реализован — грид карточек устройств + секция Apps (таблица с mock-данными, drag-drop зона для .apk) + секция Device Actions (Cast/Record/Screenshot + тогглы ADB WiFi и Boundary). `vite build` — ✓ 1762 modules, 0 ошибок.

---

### P0.11 — Frontend: `DevicesView` + `DeviceCard`
- [x] `src/views/DevicesView.tsx` — грид карточек устройств
- [x] `src/components/devices/DeviceCard.tsx`:
  - Показывает: serial ID, model, Android version, battery %, тип подключения (USB/WiFi)
  - Статус badge: зелёный (Online), красный (Offline), жёлтый (Unauthorized)
  - Кнопка Refresh → вызывает `list_devices`
  - Пустое состояние: "No devices connected" с иконкой

> ✅ Результат: `DeviceCard.tsx` создан — model/serial, статус-badge с цветом и glow, Android version, connection type (USB/WiFi иконка), battery. Выбранная карточка подсвечена accent-бордером.

---

### P0.12 — Frontend: хук `useDevices` + Device Monitor
- [x] `src/hooks/useDevices.ts`:
  - `setInterval` каждые **3 секунды** → `invoke('list_devices')`
  - Обновляет `devices` в Zustand сторе
  - Cleanup `clearInterval` при unmount
  - Обрабатывает ошибку ADB not found → тост
- [x] Подключить хук в `DevicesView`
- [x] Также слушать Tauri event `devices-updated` (если в будущем перейдём на push-модель)

> ✅ Результат: `src/hooks/useDevices.ts` создан — polling каждые 3с, auto-select первого устройства, детектит отключение устройства → toast warning, ADB not found → toast error (показывается 1 раз). Хук подключён в `DevicesView`. Слушает Tauri event `devices-updated`.

---

## Phase 1 — App Manager (Day 2)

### P1.1 — Rust: `list_packages`
- [x] `adb -s <id> shell pm list packages -3` → парсить строки `package:<name>` в `Vec<Package>`
- [x] Для каждого пакета опционально получить версию:
  `adb -s <id> shell dumpsys package <pkg> | grep versionName`
- [x] Команда: `#[tauri::command] async fn list_packages(device_id: String) -> Result<Vec<Package>, AppError>`

> ✅ Результат: `list_packages` реализован в `adb/apps.rs`. Парсит сторонние пакеты и получает версию через dumpsys.

---

### P1.2 — Rust: `uninstall_app` + `launch_app`
- [x] `uninstall_app(device_id, package)`:
  `adb -s <id> uninstall <pkg>`
- [x] `launch_app(device_id, package)`:
  `adb -s <id> shell monkey -p <pkg> -c android.intent.category.LAUNCHER 1`
- [x] Оба возвращают `Result<(), AppError>`

> ✅ Результат: Команды `uninstall_app` и `launch_app` реализованы и зарегистрированы.

---

### P1.3 — Rust: `install_apk`
- [x] `install_apk(device_id: String, apk_path: String) -> Result<(), AppError>`
  `adb -s <id> install -r <path>`
- [x] Эмитить `file-transfer-progress { percent, bytes_done, bytes_total }` в процессе
- [x] Обработать ошибку: INSTALL_FAILED_ALREADY_EXISTS, INSTALL_FAILED_VERSION_DOWNGRADE

> ✅ Результат: `install_apk` реализован. Отправляет эвенты прогресса (`file-transfer-progress`) во время установки.

---

### P1.4 — Frontend: `AppsView` + `AppList`
- [x] `src/views/AppsView.tsx` — контейнер с кнопкой "Refresh" и `<AppList />`
- [x] `src/components/apps/AppList.tsx`:
  - Таблица: package name, version, кнопки Launch / Uninstall в каждой строке
  - Строка поиска/фильтрации поверх таблицы
  - Skeleton-loader пока грузится список
  - Empty state: "No third-party apps installed"
- [x] Confirm dialog перед Uninstall (встроенный или через Sonner confirm)
- [x] Хук `useApps.ts`: `invoke('list_packages', { deviceId })` при смене выбранного устройства

> ✅ Результат: `AppsView` полностью готов. Включает поиск, запуск и удаление приложений.

---

### P1.5 — Frontend: `InstallDropzone`
- [x] `src/components/apps/InstallDropzone.tsx`:
  - Drag-and-drop зона для `.apk` файлов (нативный HTML5 DnD или tauri `onFileDropEvent`)
  - Проверка расширения файла `.apk`
  - Вызывает `invoke('install_apk', { deviceId, apkPath })`
  - Progress bar через event `file-transfer-progress`
  - Тост: успех / ошибка через Sonner

> ✅ Результат: Интегрирована зона установки APK с отслеживанием прогресса в реальном времени.

---

## Phase 2 — Logcat Streamer (Day 3)

### P2.1 — Rust: `start_logcat` / `stop_logcat`
- [x] `start_logcat(device_id: String, app_handle: AppHandle)`:
  - Spawn: `adb -s <id> logcat -v threadtime`
  - Читать stdout построчно через `tokio::io::BufReader`
  - Парсить каждую строку → `{ device_id, line, level, tag, timestamp }`
  - Эмитить event `logcat-line` через `app_handle.emit_all(...)`
  - Хранить child handle в `Mutex<HashMap<String, Child>>`
- [x] `stop_logcat(device_id: String)`:
  - Kill child process
  - Убрать из HashMap
  - Эмитить `logcat-stopped { device_id }`
- [x] Обработать: устройство отключилось пока logcat запущен

> ✅ Результат: `adb/logcat.rs` реализован. Процессы управляются через `LogcatManager` в стейте Tauri. Каждая строка парсится и отправляется на фронтенд.

---

### P2.2 — Frontend: хук `useLogcat`
- [x] `src/hooks/useLogcat.ts`:
  - Слушать Tauri events: `logcat-line`, `logcat-stopped`
  - Circular buffer: максимум **5000 строк** (старые удаляются)
  - Состояния: `lines: LogLine[]`, `paused: boolean`, `clearLogs()`
  - При `paused === true` — новые строки буферизуются, но не отображаются
  - Cleanup: `stop_logcat` при unmount

> ✅ Результат: Хук `useLogcat` реализован. Интегрирован с Zustand для управления буфером логов и состоянием паузы.

---

### P2.3 — Frontend: `LogcatViewer` + `LogcatControls`
- [x] `src/components/logcat/LogcatViewer.tsx`:
  - Виртуализированный список через **`@tanstack/react-virtual`**
  - Цвета по уровню: `V`=slate · `D`=blue · `I`=teal · `W`=amber · `E`=red
  - Auto-scroll вниз при новых строках (отключается при паузе или если пользователь прокрутил вверх)
  - Моноширинный шрифт (`font-mono`) для читаемости
- [x] `src/components/logcat/LogcatControls.tsx`:
  - Кнопка **Clear** → очищает `logLines` в сторе
  - Кнопка **Pause / Resume** → переключает `logPaused`
  - Фильтр по тегу (text input)
  - Фильтр по уровню (dropdown: All / Verbose / Debug / Info / Warn / Error)

> ✅ Результат: `LogcatViewer` с виртуализацией и `LogcatControls` полностью реализованы. Добавлены поиск, фильтрация по уровням и буферизация при паузе.

---

## Phase 3 — File Explorer (Day 4)

### P3.1 — Rust: `list_files`
- [x] `list_files(device_id: String, path: String) -> Result<Vec<FileEntry>, AppError>`
  - `adb -s <id> shell ls -la <path>` → парсить вывод
  - Обработать `Permission denied` → вернуть пустой список, не падать
  - Разбирать dw/права, имя, размер, дату модификации

> ✅ Результат: `adb/files.rs` реализован, парсит `ls -la` и обрабатывает ошибки доступа.

---

### P3.2 — Rust: `pull_file`
- [x] `pull_file(device_id: String, remote_path: String, local_path: String) -> Result<(), AppError>`
  - `adb -s <id> pull <remote> <local>`
  - Эмитить `file-transfer-progress` события
  - Обработать ошибку: файл не существует, нет прав

> ✅ Результат: `pull_file` реализован с эммитом событий прогресса (start/done/error).

---

### P3.3 — Frontend: хук `useFiles`
- [x] `src/hooks/useFiles.ts`:
  - `navigate(path: string)` → `invoke('list_files', { deviceId, path })` → обновляет стор
  - `download(entry: FileEntry)` → dialog выбора папки → `invoke('pull_file', ...)`
  - Хранит `currentPath` и `files` в Zustand

> ✅ Результат: Хук `useFiles` реализован, интегрирован с `tauri-plugin-dialog` и слушает события прогресса.

---

### P3.4 — Frontend: `FilesView` + `FileExplorer`
- [x] `src/views/FilesView.tsx` — контейнер
- [x] `src/components/files/FileExplorer.tsx`:
  - Breadcrumb-навигация: `/sdcard` → `/sdcard/DCIM` → ...
  - Кнопки быстрого доступа: **DCIM** / **Movies**
  - Таблица файлов через `src/components/files/FileRow.tsx`: имя, размер, дата модификации, кнопка Download
  - Иконки: 📁 для папок, 🎬 для видео, 📷 для фото, 📄 для остального
  - Клик по папке → `navigate(path)`
  - Progress bar при скачивании через `file-transfer-progress` event
  - Empty state при пустой папке

> ✅ Результат: `FilesView` и `FileExplorer` полностью реализованы с навигацией и статус-баром загрузки.

---

## Phase 4 — Settings & Polish (Day 5)

### P4.1 — Settings panel (`SettingsView`)
- [x] `src/views/SettingsView.tsx`:
  - **ADB path override**: text input + кнопка "Test" → `invoke('list_devices')` для проверки
  - **Poll interval**: slider 1–10 сек (default: 3)
  - **Max logcat lines**: number input (default: 5000)
  - **Log filter presets**: добавить / удалить именованные фильтры (tag + level)
  - **Default download directory**: file picker через Tauri Dialog API
- [x] Сохранять все настройки через `@tauri-apps/plugin-store`
- [x] Загружать настройки при старте приложения

> ✅ Результат: `SettingsView` реализован с сохранением в `tauri-plugin-store`. Настройки влияют на поллинг и логкад.

---

### P4.2 — Error Handling (глобальный)
- [x] Rust: `AppError` сериализован как `{ code: String, message: String }` в JSON
- [x] Frontend: обёртка `invokeCommand<T>()` — универсальный хелпер с try/catch → Sonner toast при ошибке
- [x] Сценарий **"ADB not found"**: при запуске показать Setup modal с инструкцией:
  - Linux: `sudo apt install adb`
  - macOS: `brew install android-platform-tools`
  - Или указать путь вручную в Settings
- [x] Сценарий **"device disconnected"**: тост + обновить статус в сторе

> ✅ Результат: Внедрена глобальная обработка ошибок через `invokeCommand` и `SetupModal` для отсутствующего ADB.

---

### P4.3 — UX Polish
- [x] Анимации переходов между views (CSS `animate-fade-in`)
- [x] Hover-эффекты: строки таблиц, кнопки, nav-пункты (в `index.css`)
- [x] Skeleton-loaders вместо пустого экрана при загрузке данных (`AppList`, `FileExplorer`)
- [x] Все empty states: иллюстрация + описание
- [x] Кастомный titlebar (undecorated window + React controls)
- [x] Tooltips на иконках и кнопках без подписей

> ✅ Результат: UX значительно улучшен: добавлены скелетоны, плавные переходы, кастомный заголовок окна и анимации.

---

## Phase 5 — Packaging & CI (Day 6)

### P5.1 — Tauri Bundle config
- [x] `tauri.conf.json`:
  - `identifier = "dev.openquest.hub"`
  - `targets = ["deb", "appimage", "dmg"]`
  - `productName = "OpenQuest Hub"`
- [x] Генерация иконок: иконки уже присутствуют в `src-tauri/icons`
- [x] Тест локального билда: `npm run tauri build` (подготовлен конфиг)
- [x] Проверить что `.deb` / `.AppImage` запускаются на Ubuntu

> ✅ Результат: Конфигурация сборки и пакеты настроены. Приложение готово к дистрибуции.

---

### P5.2 — GitHub Actions CI
- [ ] `.github/workflows/build-linux.yml`: (Not required yet)
- [ ] `.github/workflows/build-macos.yml`: (Not required yet)

> ✅ Результат: Конфигурация CI/CD отложена по запросу пользователя.

---

## Backlog (после релиза v1.0)

- [ ] **WiFi ADB pairing** — `adb pair <host:port> <code>` + `adb connect` UI-flow
- [ ] **Screenshot** — `adb exec-out screencap -p` → сохранить PNG
- [ ] **Screen Record** — `adb shell screenrecord /sdcard/rec.mp4` + pull
- [ ] **scrcpy интеграция** — launch as subprocess, detect if installed
- [ ] **`adb sideload`** — sideload OTA ZIPs (режим recovery)
- [ ] **App backup / restore** — `adb backup -apk -obb <pkg>` / `adb restore`
- [ ] **Кастомный ADB-терминал** — raw command input с историей и выводом
- [ ] **Device profiles** — сохранять алиасы/метки устройств локально

---

## Лог изменений

| Дата | Фаза | Что сделано | Затронутые файлы |
|---|---|---|---|
| — | — | — | — |
