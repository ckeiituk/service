# 🛡️ Outclash Service

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-windows%20%7C%20linux%20%7C%20macos-lightgrey.svg)]()

> **Кросс-платформенный системный сервис** для управления proxy-core процессами с повышенными привилегиями.
> Защищённый IPC с HMAC-SHA256, нативная интеграция с OS, работает полностью автономно.

---

Это не просто очередной service wrapper — это **надёжная прослойка между приложением и системой**, которая берёт на себя все грязные детали работы с привилегированными операциями. Вместо того чтобы каждый раз запрашивать sudo/admin права, ваше приложение общается с сервисом через безопасный IPC-канал.

### Зачем это нужно?

* **Безопасность прежде всего**
  Вместо того чтобы давать основному приложению root-доступ, весь privileged код изолирован в отдельном сервисе с минимальной поверхностью атаки. Все запросы криптографически подписаны — никаких случайных команд из непроверенных источников.

  > *Принцип наименьших привилегий: приложение работает от обычного пользователя, а сервис включается только когда нужно.*

* **Кросс-платформенность из коробки**
  Один и тот же код работает на Windows, Linux и macOS благодаря нативной интеграции с Windows Service API, systemd и launchd. Не нужно городить костыли — платформенные различия уже учтены.

  > *Написал раз, запустил везде — без дополнительных зависимостей.*

* **Легко интегрировать в свой проект**
  Простой JSON-протокол через named pipes (Windows) или Unix sockets (Linux/macOS). Библиотеки для IPC есть под любой язык — от Python до JavaScript.

  > *Не нужно разбираться в системном программировании, просто отправляй JSON и получай ответ.*

---

## Возможности

* **IPC:** Подписанные сообщения (HMAC-SHA256), автоматическая валидация timestamp
* **Управление процессами:** Запуск, остановка, мониторинг proxy-core с конфиг-валидацией
* **Установка:** Утилиты `install-service` и `uninstall-service` — одна команда, и всё готово
* **Интеграция:** Windows Service, systemd (Linux), launchd (macOS)
* **Offline-режим:** Нулевые внешние зависимости в runtime, работает без интернета

---

## Быстрый старт

<details>
<summary>🔹 Linux (systemd)</summary>

```bash
# Клонируем репозиторий
git clone https://github.com/USERNAME/outclash-service.git
cd outclash-service

# Собираем релиз
cargo build --release

# Устанавливаем сервис (требуется root)
sudo ./target/release/install-service

# Запускаем и добавляем в автозагрузку
sudo systemctl start outclash-service
sudo systemctl enable outclash-service

# Проверяем статус
systemctl status outclash-service
```

IPC endpoint: `/tmp/outclash-service.sock`

</details>

<details>
<summary>🔹 Windows (Service Manager)</summary>

```cmd
REM Собираем релиз
cargo build --release

REM Устанавливаем сервис (нужны права администратора)
target\release\install-service.exe

REM Запускаем сервис
sc start outclash-service

REM Проверяем статус
sc query outclash-service
```

IPC endpoint: `\\.\pipe\outclash-service`

</details>

<details>
<summary>🔹 macOS (launchd)</summary>

```bash
# Собираем релиз
cargo build --release

# Устанавливаем сервис (требуется root)
sudo ./target/release/install-service

# Загружаем в launchd
sudo launchctl load /Library/LaunchDaemons/io.github.outclash.service.plist

# Проверяем статус
launchctl list | grep outclash
```

IPC endpoint: `/tmp/outclash-service.sock`

</details>

---

## Архитектура

```text
┌─────────────────────────────────────────────────────┐
│                  Your Application                    │
│            (runs as regular user)                    │
└──────────────────────┬──────────────────────────────┘
                       │ JSON + HMAC-SHA256
                       ▼
         ┌─────────────────────────────┐
         │      IPC Layer              │
         │  (named pipe / unix socket) │
         └─────────────┬───────────────┘
                       │
                       ▼
         ┌─────────────────────────────┐
         │   Outclash Service          │
         │   (privileged process)      │
         │                             │
         │  ┌──────────────────────┐   │
         │  │  CoreManager         │   │
         │  │  - start_proxy()     │   │
         │  │  - stop_proxy()      │   │
         │  │  - validate_config() │   │
         │  └──────────────────────┘   │
         └─────────────┬───────────────┘
                       │
                       ▼
         ┌─────────────────────────────┐
         │     Proxy Core Process      │
         │     (mihomo/clash)          │
         └─────────────────────────────┘
```

---

## IPC Protocol

Все сообщения — это JSON с подписью:

```json
{
  "command": "StartProxy",
  "timestamp": 1234567890,
  "data": {
    "config_dir": "/path/to/config",
    "config_file": "config.yaml"
  },
  "signature": "deadbeef..."
}
```

### Доступные команды

| Команда       | Описание                          |
|---------------|-----------------------------------|
| `GetVersion`  | Версия сервиса                    |
| `GetStatus`   | Статус proxy-core                 |
| `StartProxy`  | Запустить proxy с конфигом        |
| `StopProxy`   | Остановить proxy                  |

Timestamp валидируется на стороне сервиса (окно 30 секунд) — старые запросы отбрасываются автоматически.

---

## Development

```bash
# Dev build
cargo build

# Тесты
cargo test

# Форматирование
cargo fmt

# Линтер
cargo clippy -- -D warnings
```

### Структура проекта

```text
src/
├── main.rs          # Entry point, logging
├── install.rs       # Утилита установки
├── uninstall.rs     # Утилита удаления
└── service/
    ├── mod.rs       # Платформенные адаптеры
    ├── core.rs      # Lifecycle управление proxy
    ├── ipc.rs       # IPC server + crypto
    ├── data.rs      # State management
    └── process.rs   # Spawn/kill утилиты
```

### Кросс-компиляция

```bash
# Windows (из Linux/Mac)
cargo build --release --target x86_64-pc-windows-msvc

# Linux ARM (напр. Raspberry Pi)
cargo build --release --target aarch64-unknown-linux-gnu

# macOS Apple Silicon
cargo build --release --target aarch64-apple-darwin
```

---

## Roadmap

- [ ] HTTP REST API для управления (опциональная альтернатива IPC)
- [ ] WebSocket для real-time статуса и логов
- [ ] Поддержка множественных proxy-core экземпляров
- [ ] Метрики и мониторинг (Prometheus exporter)
- [ ] GUI для управления сервисом

---

## Зачем ещё один service wrapper?

Большинство решений либо слишком монолитные (всё в одном процессе), либо требуют сложной настройки. Outclash Service — это **минимализм + безопасность**:

- Один бинарник, никаких runtime-зависимостей
- Понятный протокол, простая интеграция
- Production-ready security из коробки
- Работает локально, нет облачных зависимостей

Это foundation, на котором можно строить что угодно — от desktop приложений до системных агентов.

---

## Логирование

По умолчанию выключено (zero overhead). Для включения:

1. Откройте `src/main.rs`
2. Раскомментируйте `ENABLE_LOGGING.store(true, Ordering::Relaxed)`
3. Пересоберите
4. Логи пишутся в `outclash-service.log` рядом с бинарником

---

## Удаление сервиса

```bash
# Linux/macOS
sudo ./target/release/uninstall-service

# Windows (cmd as Administrator)
uninstall-service.exe
```

---

## 📝 Лицензия

GPL-3.0. Форкайте, модифицируйте, используйте — только не забудьте поставить ⭐ репозиторию!

---

Made with 🦀 Rust and ☕

