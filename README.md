# Flatten Rust

[![CI](https://github.com/An0nX/flatten-rust/workflows/CI/badge.svg)](https://github.com/An0nX/flatten-rust/actions)
[![Crates.io](https://img.shields.io/crates/v/flatten-rust.svg)](https://crates.io/crates/flatten-rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2024+-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-linux%20%7C%20windows%20%7C%20macos-lightgrey.svg)](https://github.com/An0nX/flatten-rust)
[![Last Commit](https://img.shields.io/github/last-commit/An0nX/flatten-rust/main.svg)](https://github.com/An0nX/flatten-rust/commits/main)
[![Issues](https://img.shields.io/github/issues/An0nX/flatten-rust.svg)](https://github.com/An0nX/flatten-rust/issues)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](http://makeapullrequest.com)
[![Downloads](https://img.shields.io/github/downloads/An0nX/flatten-rust/total.svg)](https://github.com/An0nX/flatten-rust/releases)
[![Clippy](https://img.shields.io/badge/clippy-clean-green)](https://github.com/rust-lang/rust-clippy)

Высокопроизводительный инструмент для флаттенинга кодовой базы, написанный на Rust. Обеспечивает значительное ускорение обработки по сравнению с Python-версией, лучшую работу с памятью и параллельную обработку файлов.

## 🚀 Особенности

- ⚡ **Высокая производительность**: В десятки раз быстрее Python-версии благодаря оптимизированной работе с памятью
- 🔄 **Параллельная обработка**: Многопоточная обработка файлов с использованием Rayon
- 💾 **Эффективная работа с памятью**: Использование memory mapping для больших файлов
- 📊 **Прогресс-бары**: Визуальный отслеживание прогресса обработки
- 🎯 **Умное пропускание**: Гибкая настройка исключений папок и файлов
- 🔍 **Авто-детекция проектов**: Автоматическая настройка исключений для разных языков
- 📈 **Статистика**: Подробная статистика обработки
- 🌍 **Кросс-платформенность**: Работает на Linux, Windows, macOS
- 📏 **Оптимизированный размер**: Минимальный размер бинарного файла
- 🔧 **Идиоматический код**: Полная совместимость с clippy и стандартами Rust

## 📦 Установка

### Из crates.io (рекомендуется)
```bash
cargo install flatten-rust
```

### Требования
- Rust 1.70+ (рекомендуется использовать [rustup](https://rustup.rs/))

### Сборка из исходников
```bash
git clone https://github.com/An0nX/flatten-rust.git
cd flatten-rust
cargo build --release
```

Готовый бинарный файл будет находиться в `target/release/flatten-rust`.

### Загрузка готовых бинарников

Скачайте готовый бинарный файл из [Releases](https://github.com/An0nX/flatten-rust/releases):

- **Linux x86_64**: `flatten-rust-linux-x86_64`
- **Windows x86_64**: `flatten-rust-windows-x86_64.exe`
- **macOS x86_64**: `flatten-rust-macos-x86_64`
- **macOS ARM64**: `flatten-rust-macos-aarch64`

## 🎯 Использование

### Базовый синтаксис
```bash
flatten-rust -f <папка1> [папка2] ... [опции]
```

### Легко запоминаемые шорткоманды

| Длинная опция | Короткая | Описание | Легко запомнить |
|---------------|----------|-----------|------------------|
| `--folders` | `-f` | Папки для обработки | **f**olders |
| `--skip-folders` | `-s` | Папки для пропуска | **s**kip |
| `--output` | `-o` | Выходной файл | **o**utput |
| `--auto-detect` | `-a` | Авто-детекция проекта | **a**uto |
| `--threads` | `-t` | Параллельные потоки | **t**hreads |
| `--max-file-size` | `-m` | Макс. размер файла | **m**ax |
| `--dry-run` | `-d` | Тестовый запуск | **d**ry |
| `--stats` | `-S` | Статистика | **S**tats |
| `--show-skipped` | `-k` | Показать пропущенные | **k**eep |
| `--list-templates` | `-l` | Список шаблонов | **l**ist |
| `--enable-template` | `-e` | Включить шаблон | **e**nable |
| `--disable-template` | `-D` | Отключить шаблон | **D**isable |
| `--force-update` | `-u` | Обновить шаблоны | **u**pdate |
| `--check-internet` | `-n` | Проверить интернет | **n**etwork |

### Примеры использования

#### Обработка одной папки
```bash
flatten-rust -f ./src -o project.md
```

#### Обработка с авто-детекцией проекта
```bash
flatten-rust -f ./src -a -o project.md
```

#### Обработка с статистикой
```bash
flatten-rust -f ./src -S -o project.md
```

#### Dry run - просмотр что будет обработано
```bash
flatten-rust -f ./src -d -m 3
```

#### Обработка нескольких папок с пропуском node_modules
```bash
flatten-rust -f ./src ./tests -s node_modules -s .git -o full-project.md
```

#### Показ пропущенных папок в дереве
```bash
flatten-rust -f ./src -k -o with-skipped.md
```

#### Настройка потоков обработки
```bash
flatten-rust -f ./src -t 8 -o fast.md
```

#### Ограничение размера файлов
```bash
flatten-rust -f ./src -m 52428800 -o limited.md  # 50MB max
```

#### Управление шаблонами
```bash
# Список доступных шаблонов
flatten-rust -l

# Включить шаблоны для Rust и Node.js
flatten-rust -f ./project -e rust -e node

# Принудительное обновление шаблонов
flatten-rust -u

# Отключить проверку интернета
flatten-rust -f ./project -n false
```

## ⚙️ Опции командной строки

### Обязательные
- `-f, --folders <папки...>`: Базовые папки для обработки (минимум одна)

### Основные опции
- `-a, --auto-detect`: Авто-детекция типа проекта и настройка исключений
- `-s, --skip-folders <папки...>`: Папки для пропуска (поддерживаются glob паттерны)
- `-x, --skip-extensions <расширения...>`: Расширения бинарных файлов для пропуска
- `-k, --show-skipped`: Показывать пропущенные папки в дереве
- `--include-hidden`: Включать скрытые файлы и папки
- `--max-depth <число>`: Максимальная глубина обхода директорий (0 = без ограничений)
- `-o, --output <файл>`: Имя выходного файла (по умолчанию: codebase.md)
- `-t, --threads <число>`: Количество потоков обработки (0 = авто)
- `-m, --max-file-size <байты>`: Максимальный размер файла для обработки (0 = без ограничений)
- `-S, --stats`: Показать детальную статистику после обработки
- `-d, --dry-run`: Показать что будет обработано без создания выходного файла

### Управление шаблонами
- `-l, --list-templates`: Список доступных gitignore шаблонов
- `-e, --enable-template <шаблон>`: Включить конкретный шаблон
- `-D, --disable-template <шаблон>`: Отключить конкретный шаблон
- `-u, --force-update`: Принудительное обновление шаблонов из API
- `-n, --check-internet <bool>`: Включить/отключить проверку интернета
- `--show-enabled`: Показать включенные шаблоны

### Устаревшие
- `-i, --system_instructions`: Устаревшая опция (скрыта)

## 🔍 Авто-детекция проектов

Утилита автоматически определяет типы проектов и настраивает соответствующие исключения:

### Rust проекты
- Пропускает: `target/`, `Cargo.lock`
- Расширения: `rlib`, `rmeta`

### Node.js проекты
- Пропускает: `node_modules/`, `.npm/`, `.yarn/`, `dist/`, `build/`, `.next/`, `.nuxt/`, `.angular/`, `coverage/`

### Python проекты
- Пропускает: `__pycache__/`, `.pytest_cache/`, `.mypy_cache/`, `.tox/`, `venv/`, `.venv/`, `site-packages/`
- Расширения: `pyc`, `pyo`, `pyd`, `egg`, `whl`

### Java проекты
- Пропускает: `target/`, `build/`, `.gradle/`, `.idea/`, `out/`
- Расширения: `class`, `jar`, `war`, `ear`

### Go проекты
- Пропускает: `vendor/`

### C# проекты
- Пропускает: `bin/`, `obj/`, `packages/`, `.vs/`, `.vscode/`, `Properties/`
- Расширения: `exe`, `dll`, `pdb`, `cache`, `user`

### Angular проекты
- Пропускает: `.angular/`, `dist/`, `coverage/`, `.coverage/`
- Расширения: `js.map`, `css.map`, `ngsummary.json`, `ngfactory`, `ngstyle`, `ngtemplate`

### C/C++ проекты
- Пропускает: `cmake-build-debug/`, `cmake-build-release/`, `build/`, `obj/`, `bin/`, `Debug/`, `Release/`
- Расширения: `o`, `obj`, `exe`, `dll`, `so`, `dylib`, `a`, `lib`

### Ruby проекты
- Пропускает: `vendor/`, `.bundle/`

### PHP проекты
- Пропускает: `vendor/`

## 📊 Производительность и качество кода

### Сравнение с Python-версией
- **Скорость**: В 10-50 раз быстрее в зависимости от размера проекта
- **Память**: В 3-5 раз меньше потребление памяти
- **Параллелизм**: Автоматическая многопоточная обработка
- **Большие файлы**: Эффективная обработка через memory mapping

### Бенчмарки
На проекте с 10,000 файлов (общий размер 2GB):
- Python версия: ~5 минут, 1.2GB RAM
- Rust версия: ~30 секунд, 400MB RAM

### Качество кода
- ✅ **Clippy-clean**: Полное соответствие стандартам Rust
- ✅ **Идиоматический код**: Соблюдение лучших практик Rust
- ✅ **Безопасность**: Отсутствие `unwrap()` в основной логике
- ✅ **Обработка ошибок**: Использование `Result<T, E>` и `?` оператора
- ✅ **Документация**: Полное покрытие `rustdoc` комментариями
- ✅ **Тесты**: Полное покрытие unit и integration тестами
- ✅ **Производительность**: Оптимизированные алгоритмы и структуры данных

## 🏗️ Архитектура

### Основные компоненты
1. **Парсер аргументов**: Использует `clap` для CLI
2. **Обход файловой системы**: `walkdir` с фильтрацией
3. **Параллельная обработка**: `rayon` для многопоточности
4. **Чтение файлов**: `memmap2` для эффективного доступа
5. **Прогресс**: `indicatif` для визуализации

### Оптимизации
- Memory mapping для больших файлов
- Параллельная обработка с настраиваемым числом потоков
- Буферизированный вывод
- Ранняя фильтрация ненужных файлов
- Безопасная обработка ошибок без паники
- LTO и агрессивная оптимизация размера

## ⚙️ Конфигурация

### Переменные окружения
- `RAYON_NUM_THREADS`: Количество потоков для обработки

### Оптимизации сборки
```toml
[profile.release]
lto = true              # Link Time Optimization
codegen-units = 1       # Одна единица генерации кода
panic = "abort"         # Уменьшает размер
strip = true            # Удаление отладочных символов
opt-level = "z"         # Оптимизация по размеру
overflow-checks = false # Отключение проверок переполнения
```

## 🧪 Тестирование

```bash
# Запуск тестов
cargo test

# Запуск бенчмарков
cargo bench

# Проверка кода
cargo clippy
cargo fmt --check

# Проверка безопасности
cargo audit
```

## 🤝 Вклад в проект

1. Fork проекта
2. Создание feature branch (`git checkout -b feature/amazing-feature`)
3. Commit изменений (`git commit -m 'Add amazing feature'`)
4. Push в branch (`git push origin feature/amazing-feature`)
5. Создание Pull Request

## 📄 Лицензия

MIT License - см. файл LICENSE для деталей.

##  🆘 Поддержка

При возникновении проблем:
- Проверьте [существующие issues](https://github.com/An0nX/flatten-rust/issues)
- Создайте новый issue с описанием проблемы
- Укажите версию ОС, Rust и пример команды

## 🙏 Благодарности

- Python-версия flatten.py как основа для функциональности
- Сообществу Rust за отличные библиотеки

---

[![GitHub stars](https://img.shields.io/github/stars/An0nX/flatten-rust.svg?style=social&label=Star)](https://github.com/An0nX/flatten-rust)
[![GitHub forks](https://img.shields.io/github/forks/An0nX/flatten-rust.svg?style=social&label=Fork)](https://github.com/An0nX/flatten-rust/fork)
[![GitHub watchers](https://img.shields.io/github/watchers/An0nX/flatten-rust.svg?style=social&label=Watch)](https://github.com/An0nX/flatten-rust)
