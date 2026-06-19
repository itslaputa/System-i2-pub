# System-I2

[English](README.md) | [Русский](README.ru.md)

Персональный настольный трекер времени для записи работы, организации задач по проектам и категориям, а также анализа того, куда реально уходит время.

System-I2 работает локально: задачи, комментарии, проекты, категории и аналитика остаются на вашей машине. В приложении нет аккаунтов, облачной синхронизации и внешнего сервера.

[Скачать](#скачать) · [Приватность](#приватность-и-данные) · [Для разработчиков](#для-разработчиков)

![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?style=flat-square)
![Rust](https://img.shields.io/badge/Rust-backend-000000?style=flat-square)
![SolidJS](https://img.shields.io/badge/SolidJS-frontend-2C4F7C?style=flat-square)
![SQLite](https://img.shields.io/badge/SQLite-local%20storage-003B57?style=flat-square)
![Local-first](https://img.shields.io/badge/data-local--first-3C873A?style=flat-square)

## Скачать

Последний публичный релиз: [System-I2 v1](https://github.com/itslaputa/System-i2-pub/releases/tag/v1)

| Платформа | Рекомендуемая загрузка | Другие форматы |
| --- | --- | --- |
| macOS Apple Silicon | [DMG](https://github.com/itslaputa/System-i2-pub/releases/download/v1/System-I2-1.5.1-darwin-aarch64.dmg) | [архив приложения](https://github.com/itslaputa/System-i2-pub/releases/download/v1/System-I2-1.5.1-darwin-aarch64.app.tar.gz) |
| Windows x64 | [MSI-установщик](https://github.com/itslaputa/System-i2-pub/releases/download/v1/System-I2-1.5.1-windows-x64.msi) | [EXE-установщик](https://github.com/itslaputa/System-i2-pub/releases/download/v1/System-I2-1.5.1-windows-x64.exe) |
| Linux x64 | [AppImage](https://github.com/itslaputa/System-i2-pub/releases/download/v1/System-I2-1.5.1-linux-amd64.AppImage) | [DEB](https://github.com/itslaputa/System-i2-pub/releases/download/v1/System-I2-1.5.1-linux-amd64.deb), [RPM](https://github.com/itslaputa/System-i2-pub/releases/download/v1/System-I2-1.5.1-linux-x86_64.rpm) |

Другие варианты: [последний релиз](https://github.com/itslaputa/System-i2-pub/releases/latest) · [все релизы](https://github.com/itslaputa/System-i2-pub/releases)

Текущие сборки не подписаны. macOS, Windows и некоторые Linux-окружения могут показать предупреждение безопасности при первом запуске. Основная проверенная платформа сейчас macOS; Windows и Linux собираются автоматически и перед серьёзным использованием требуют ручной проверки на целевой системе.

## Первый запуск

При первом запуске System-I2 спросит, где хранить локальные данные:

- создать новую папку данных внутри стандартной папки приложения;
- создать новую папку данных в выбранном месте;
- подключить существующую папку данных.

Корректная папка данных содержит:

- `tasks.sqlite3`
- `task_categories.json`
- `task_category_change_log.log`
- `project_categories.json`

## Приватность и данные

System-I2 не загружает ваши задачи в облако. В публичном приложении нет аккаунта, синхронизации и удалённой аналитики.

Репозиторий не содержит личных данных. Локальные SQLite-файлы, пользовательские настройки, бэкапы и результаты сборки игнорируются и не должны попадать в коммиты.

## Статус проекта

System-I2 — рабочее локальное настольное приложение с публичным исходным кодом и неподписанными сборками.

- Основная проверенная платформа: macOS.
- Цели сборки через Tauri: macOS, Windows, Linux.
- Подпись и нотариальное заверение для macOS: пока не настроены.
- Модель данных: локальные SQLite/JSON-файлы.

## Для разработчиков

Большинству пользователей достаточно скачать готовую сборку выше. Разделы ниже нужны тем, кто хочет собрать, проверить или доработать приложение.

<details>
<summary>Запуск из исходников</summary>

```bash
npm install
npm run tauri dev
```

</details>

<details>
<summary>Локальная сборка</summary>

```bash
npm run tauri build -- --bundles app
```

Заметки по сборке, требованиям платформ и проверкам релиза находятся в [docs/BUILD.ru.md](docs/BUILD.ru.md).

</details>

<details>
<summary>Проверки разработки</summary>

Фронтенд:

```bash
npm run test:frontend
npx tsc --noEmit --pretty false
npm run build
```

Бэкенд:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

</details>

<details>
<summary>Структура репозитория</summary>

- `src/`: интерфейс на SolidJS.
- `src/services/tauri/`: TypeScript-мост к командам Rust/Tauri.
- `src-tauri/src/`: бэкенд на Rust.
- `src-tauri/src/storage/`: подключение SQLite, схема, валидация и работа с путями данных.
- `src-tauri/src/runtime/`: первый запуск, проверка папки данных, подключение/создание данных и резервная копия.
- `tests/frontend/`: модульные тесты интерфейса.
- Rust-тесты лежат рядом со своими бэкенд-доменами в `src-tauri/src/**/tests/`.

Более глубокие правила разработки: [agents_pub/agents.ru.md](agents_pub/agents.ru.md).

</details>

## Документация

- [Данные приложения](docs/DATA.ru.md)
- [Руководство по сборке](docs/BUILD.ru.md)
- [Руководство по релизам](docs/RELEASE.ru.md)
- [Публичное руководство для агентов](agents_pub/agents.ru.md)
- [Документация на английском](README.md)
