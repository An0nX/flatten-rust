# Flatten Rust

[![CI](https://github.com/An0nX/flatten-rust/workflows/CI/badge.svg)](https://github.com/An0nX/flatten-rust/actions)
[![Crates.io](https://img.shields.io/crates/v/flatten-rust.svg)](https://crates.io/crates/flatten-rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-linux%20%7C%20windows%20%7C%20macos-lightgrey.svg)](https://github.com/An0nX/flatten-rust)
[![Last Commit](https://img.shields.io/github/last-commit/An0nX/flatten-rust/main.svg)](https://github.com/An0nX/flatten-rust/commits/main)
[![Issues](https://img.shields.io/github/issues/An0nX/flatten-rust.svg)](https://github.com/An0nX/flatten-rust/issues)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](http://makeapullrequest.com)
[![Downloads](https://img.shields.io/crates/d/flatten-rust.svg)](https://crates.io/crates/flatten-rust)

Высокопроизводительный инструмент для флаттенинга кодовой базы, написанный на Rust. Обеспечивает значительное ускорение обработки по сравнению с Python-версией, лучшую работу с памятью и параллельную обработку файлов.

## 🚀 Особенности

- ⚡ **Высокая производительность**: В десятки раз быстрее Python-версии благодаря оптимизированной работе с памятью
- 🔄 **Параллельная обработка**: Многопоточная обработка файлов с использованием Rayon
- 💾 **Эффективная работа с памятью**: Использование memory mapping для больших файлов
- 📊 **Прогресс-бары**: Визуальный отслеживание прогресса обработки
- 🎯 **Умное пропуска**: Гибкая настройка исключений папок и файлов
- 🔍 **Авто-детекция проектов**: Автоматическая настройка исключений для разных языков
- 📈 **Статистика**: Подробная статистика обработки
- 🔒 **Безопасность**: Проверка размеров файлов и безопасная обработка ошибок
- 🌍 **Кросс-платформенность**: Работает на Linux, Windows, macOS
- 📏 **Оптимизированный размер**: Всего 816KB бинарный файл
- 🛡️ **Безопасная обработка**: Обработка ошибок без паники

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

- **Linux x86_64**: `flatten-rust-linux-x86_64.gz`
- **Linux ARM64**: `flatten-rust-linux-aarch64.gz` 
- **Windows x86_64**: `flatten-rust-windows-x86_64.exe.zip`
- **macOS x86_64**: `flatten-rust-macos-x86_64.gz`
- **macOS ARM64**: `flatten-rust-macos-aarch64.gz`

## 🎯 Использование

### Базовый синтаксис
```bash
flatten-rust --folders <папка1> [папка2] ... [опции]
```

### Примеры использования

#### Обработка одной папки
```bash
flatten-rust --folders ./src --output project.md
```

#### Обработка с авто-детекцией проекта
```bash
flatten-rust --folders ./src --auto-detect --output project.md
```

#### Обработка с статистикой
```bash
flatten-rust --folders ./src --stats --output project.md
```

#### Dry run - просмотр что будет обработано
```bash
flatten-rust --folders ./src --dry-run --max-depth 3
```

#### Обработка нескольких папок с пропуском node_modules
```bash
flatten-rust --folders ./src ./tests --skip-folders node_modules .git --output full-project.md
```

#### Показ пропущенных папок в дереве
```bash
flatten-rust --folders ./src --show-skipped --output with-skipped.md
```

#### Настройка потоков обработки
```bash
flatten-rust --folders ./src --threads 8 --output fast.md
```

#### Ограничение размера файлов
```bash
flatten-rust --folders ./src --max-file-size 52428800 --output limited.md  # 50MB max
```

## ⚙️ Опции командной строки

### Обязательные
- `--folders <папки...>`: Базовые папки для обработки (минимум одна)

### Опциональные
- `--auto-detect`: Авто-детекция типа проекта и настройка исключений
- `--skip-folders <папки...>`: Папки для пропуска (поддерживаются glob паттерны)
- `--skip-extensions <расширения...>`: Расширения бинарных файлов для пропуска
- `--show-skipped`: Показывать пропущенные папки в дереве
- `--include-hidden`: Включать скрытые файлы и папки
- `--max-depth <число>`: Максимальная глубина обхода директорий (0 = без ограничений)
- `--output <файл>`: Имя выходного файла (по умолчанию: codebase.md)
- `--threads <число>`: Количество потоков обработки (0 = авто)
- `--max-file-size <байты>`: Максимальный размер файла для обработки (0 = без ограничений)
- `--stats`: Показать детальную статистику после обработки
- `--dry-run`: Показать что будет обработано без создания выходного файла
- `--system_instructions`: Показать системные инструкции

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

## 📊 Производительность

### Сравнение с Python-версией
- **Скорость**: В 10-50 раз быстрее в зависимости от размера проекта
- **Память**: В 3-5 раз меньше потребление памяти
- **Параллелизм**: Автоматическая многопоточная обработка
- **Большие файлы**: Эффективная обработка через memory mapping

### Бенчмарки
На проекте с 10,000 файлов (общий размер 2GB):
- Python версия: ~5 минут, 1.2GB RAM
- Rust версия: ~30 секунд, 400MB RAM

### Размер бинарного файла
- **Оптимизированный**: 816KB
- **Без оптимизации**: 1.8MB
- **Экономия**: 55% уменьшение размера

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

## 🔧 Совместимость

- **ОС**: Linux, Windows, macOS
- **Архитектуры**: x86_64, ARM64, другие поддерживаемые Rust
- **Версия Rust**: 1.70+

## 📚 История версий

### v0.2.0
- Добавлена авто-детекция проектов
- Расширены списки исключений для популярных языков
- Добавлена статистика обработки
- Добавлен dry-run режим
- Улучшенная обработка скрытых файлов
- Оптимизация производительности
- Уменьшение размера бинарного файла до 816KB

### v0.1.0
- Первоначальный релиз
- Базовая функциональность флаттенинга
- Параллельная обработка
- Прогресс-бары
- Кросс-платформенность

## 🆘 Поддержка

При возникновении проблем:
1. Проверьте [Issues](https://github.com/An0nX/flatten-rust/issues)
2. Создайте новый issue с подробным описанием
3. Укажите версию ОС, Rust и размер проекта

## 🙏 Благодарности

- Python-версия flatten.py как основа для функциональности
- Сообществу Rust за отличные библиотеки
- Пользователям за фидбэк и предложения

---

[![GitHub stars](https://img.shields.io/github/stars/An0nX/flatten-rust.svg?style=social&label=Star)](https://github.com/An0nX/flatten-rust)
[![GitHub forks](https://img.shields.io/github/forks/An0nX/flatten-rust.svg?style=social&label=Fork)](https://github.com/An0nX/flatten-rust/fork)
[![GitHub watchers](https://img.shields.io/github/watchers/An0nX/flatten-rust.svg?style=social&label=Watch)](https://github.com/An0nX/flatten-rust)