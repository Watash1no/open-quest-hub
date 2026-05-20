# OpenQuest Hub TUI - Задачи

## Фаза 1: Инфраструктура

- [ ] Создать структуру Rust проекта для TUI (`cargo new openquest-tui`)
- [ ] Добавить зависимости в Cargo.toml: ratatui, tokio, serde, crossterm
- [ ] Настроить логирование (tracing)
- [ ] Создать базовую структуру приложения (app state, run loop)

## Фаза 2: ADB Integration

- [ ] Скопировать/адаптировать ADB модули из `src-tauri/src/adb/`
- [ ] Реализовать `adb::find_adb()` - поиск ADB бинарника
- [ ] Реализовать `adb::devices::list_devices()` - получение списка устройств
- [ ] Реализовать пулы для всех команд (apps, files, logcat, controls)

## Фаза 3: Layout и Навигация

- [ ] Создать layout: sidebar (слева) + main area (справа)
- [ ] Реализовать навигацию lazygit-style:
  - Tab для переключения между sidebar/main
  - Стрелки для навигации по спискам
  - Enter для подтверждения выбора
  - Поддержка мыши (click)
- [ ] Sidebar: 5 пунктов (Devices, Apps, Files, Logcat, Settings)
- [ ] Status bar ( снизу): выбранное устройство, статус подключения

## Фаза 4: Views

### Devices View
- [ ] Список устройств с статусом (Online 🟢, Unauthorized 🔴, Offline ⚫)
- [ ] Информация о выбранном устройстве (модель, Android версия, батарея)
- [ ] Actions панель: Refresh, Screenshot, Record, WiFi ADB, Boundary toggle

### Apps View
- [ ] Список установленных пакетов (Package.name, label, version)
- [ ] Поиск/фильтр пакетов
- [ ] Actions: Launch (Enter), Stop, Uninstall
- [ ] Установка APK+OBB через встроенный файловый браузер

### Files View
- [ ] Встроенный файловый браузер (/sdcard)
- [ ] Навигация: cd, cd .., списки файлов
- [ ] Множественный выбор: Space = mark, Enter = confirm
- [ ] Скачивание файлов (pull)
- [ ] Быстрые ссылки: Screenshots, VideoShots, Oculus, DCIM

### Logcat View
- [ ] Реаль-time стрим логов
- [ ] Фильтры по уровню (V/D/I/W/E/F)
- [ ] Фильтр по тегу (search)
- [ ] Очистка (Ctrl+C)
- [ ] Экспорт в файл

### Settings View
- [ ] ADB Path override
- [ ] Device polling interval (slider 1-10 сек)
- [ ] Max logcat lines
- [ ] Download directory (file picker)
- [ ] ADB Status диагностика

## Фаза 5: Device Actions

- [ ] Screenshot - `adb shell screencap`
- [ ] Record video - `adb shell screenrecord`
- [ ] Toggle Boundary (Quest) - `adb shell setprop guardian.system_switch 0/1`
- [ ] WiFi ADB setup - `adb tcpip 5555`, подключение по IP
- [ ] Delete remote media

## Фаза 6: UX и Polish

- [ ] Notification/Toast система в терминале
- [ ] Progress bars для долгих операций
- [ ] Keyboard shortcuts (справка = ?)
- [ ] Обработка ошибок (async Result -> UI feedback)
- [ ] Graceful shutdown (kill logcat processes, adb kill-server)

## Фаза 7: Сборка

- [ ] Cargo build --release
- [ ] Тестирование всех вьюх
- [ ] Чистка кода
- [ ] README для TUI версии

---

## Технические решения

- **Фреймворк**: ratatui (см. вопросы)
- **Навигация**: lazygit-style (Tab + arrows + mouse)
- **File picker**: встроенный браузер с множественным выбором (Space + Enter)
- **Скринкастинг**: вырезан (см. вопросы)
- **ADB**: переиспользовать логику из src-tauri/adb/*.rs