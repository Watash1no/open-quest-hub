# OpenQuest TUI — Todo

## ✅ Сделано

### Ядро
- Rust + ratatui 0.28 + crossterm 0.28
- Tabs навигация (Devices / Apps / Files / Logcat / Settings)
- Sidebar (список) + main area (детали) + status bar
- Arrow keys навигация, Enter, Tab/BackTab

### ADB модули
- `devices.rs` — `adb devices -l`, парсинг статуса + `DeviceStatus` enum
- `apps.rs` — `adb shell pm list packages -3` + `uninstall_app` + `launch_app`
- `files.rs` — `adb shell ls -la` + навигация по папкам + `pull_file()`
- `logcat.rs` — BufReader стриминг в отдельном треде + `clear_logcat()`

### Управление (полный список)
- Tab / BackTab — переключение вкладок
- ↑ / ↓ — навигация по спискам / скролл logcat
- PgUp / PgDn — скролл logcat ±20 строк
- Enter — выбор устройства / вход в папку / запуск logcat / запуск приложения
- Esc — выйти из папки на уровень выше
- r — обновить текущий вид
- p — pull файла → ~/Downloads (Files)
- u — uninstall приложения (Apps) с confirm диалогом
- d — delete файла/папки с confirm диалогом
- c — очистить logcat
- s — toggle auto-scroll logcat
- ? — help popup
- q / Ctrl+C — выход

### UX / Интерфейс
- `ListState` из ratatui — нативное выделение строк (вместо ручного `●/○`)
- Цвета статуса устройств (● Online=зелёный, ◐ Unauthorized=жёлтый, ○ Offline=красный)
- Цвета logcat по уровню (E=красный, W=жёлтый, I=белый, D=голубой, V=серый)
- Notification/toast исчезает через 3 сек
- Confirm диалог (y/n/Esc) перед uninstall/delete
- Help popup по `?`
- Breadcrumb текущего пути в заголовке Files
- Авто-выбор первого online устройства при старте
- Авто-скролл logcat с toggle (`s`)
- Скролл logcat PgUp/PgDn
- Реальное имя модели через getprop (ro.product.model + ro.build.version.release)
- Нет warnings в коде

---

## 🚧 Оставшийся функционал

### Devices
- [ ] Battery level в Device Info (`adb shell dumpsys battery | grep level`)
- [ ] Serial number

### Apps
- [ ] Поиск/фильтрация по имени пакета (ввод текста)
- [ ] Toggle: все пакеты / только сторонние / системные

### Files
- [ ] Push файла на устройство (`adb push`)
- [ ] Дата модификации в списке файлов (парсинг `ls -la`)
- [ ] Прогресс-бар при pull больших файлов

### Logcat
- [ ] Фильтрация по тегу / уровню (ввод текста)
- [ ] Экспорт в файл (сохранить буфер)

### Settings
- [ ] Сохранение конфига в `~/.config/openquest-tui/config.toml`
- [ ] Настройки: путь adb, интервал poll, лимит строк

### Архитектура
- [ ] Разбить `main.rs` (600+ строк) на `app.rs` + `ui.rs`
- [ ] `serde` + `toml` для конфига

---

## Запуск
```bash
cd openquest-tui
cargo run --release
```

## Структура проекта
```
openquest-tui/
├── Cargo.toml
└── src/
    ├── main.rs          # App state, event loop, draw (всё в одном файле)
    └── adb/
        ├── mod.rs
        ├── devices.rs   # DeviceStatus enum, getprop model/android
        ├── apps.rs      # list + uninstall + launch
        ├── files.rs     # browse + pull_file
        └── logcat.rs    # BufReader streaming + clear
```