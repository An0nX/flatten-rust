# Использование Flatten Rust

Это руководство поможет вам эффективно использовать flatten-rust для анализа и документирования кодовых баз.

## Быстрый старт

### Базовое использование
```bash
# Анализ текущей директории
./flatten-rust --folders . --output codebase.md

# Анализ нескольких директорий
./flatten-rust --folders src tests docs --output full_project.md

# С пропуском ненужных папок
./flatten-rust --folders . --skip-folders node_modules .git target --output clean.md
```

## Продвинутое использование

### Оптимизация производительности
```bash
# Увеличение количества потоков для больших проектов
./flatten-rust --folders . --threads 8 --output fast.md

# Ограничение размера файлов для экономии памяти
./flatten-rust --folders . --max-file-size 52428800 --output limited.md  # 50MB

# Пропуск бинарных файлов
./flatten-rust --folders . --skip-extensions exe dll so img pdf --output text_only.md
```

### Визуализация структуры
```bash
# Показывать пропущенные папки в дереве
./flatten-rust --folders . --show-skipped --output with_skipped.md

# Только структура без содержимого (быстро)
./flatten-rust --folders . --output structure.md --max-file-size 1
```

## Примеры использования

### Анализ Rust проекта
```bash
./flatten-rust \
  --folders src tests examples \
  --skip-folders target .git \
  --skip-extensions bin dll so \
  --output rust_project.md \
  --threads 4
```

### Анализ JavaScript проекта
```bash
./flatten-rust \
  --folders src public tests \
  --skip-folders node_modules .next dist build \
  --skip-extensions jpg png gif ico \
  --output js_project.md \
  --show-skipped
```

### Анализ Python проекта
```bash
./flatten-rust \
  --folders src tests docs \
  --skip-folders __pycache__ .pytest_cache venv .venv \
  --skip-extensions pyc pyo so \
  --output python_project.md
```

## Оптимизация для больших проектов

### Эффективный анализ монорепозиториев
```bash
# Анализ только важных модулей
./flatten-rust \
  --folders packages/core packages/utils packages/api \
  --skip-folders node_modules .git dist build coverage \
  --output mono_repo_core.md \
  --threads 8

# Полный анализ с пропуском бинарных данных
./flatten-rust \
  --folders . \
  --skip-folders node_modules .git target dist build coverage \
  --skip-extensions jpg png gif mp4 avi pdf zip tar gz \
  --max-file-size 104857600 \
  --output full_mono_repo.md \
  --threads 12
```

### Работа с ограниченными ресурсами
```bash
# Минимальное потребление памяти
./flatten-rust \
  --folders . \
  --max-file-size 10485760 \
  --threads 1 \
  --output minimal.md

# Быстрый анализ только структуры
./flatten-rust \
  --folders . \
  --max-file-size 100 \
  --output structure_only.md
```

## Интеграция с рабочими процессами

### Автоматическая документация
```bash
#!/bin/bash
# generate_docs.sh

PROJECT_DIR="/path/to/project"
OUTPUT_DIR="/path/to/docs"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

./flatten-rust \
  --folders "$PROJECT_DIR/src" "$PROJECT_DIR/docs" \
  --skip-folders node_modules .git target \
  --output "$OUTPUT_DIR/codebase_$TIMESTAMP.md" \
  --threads 4

echo "Documentation generated: $OUTPUT_DIR/codebase_$TIMESTAMP.md"
```

### CI/CD интеграция
```yaml
# .github/workflows/docs.yml
name: Generate Documentation

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  docs:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v2
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        
    - name: Build flatten-rust
      run: |
        cd path/to/flatten-rust
        cargo build --release
        
    - name: Generate documentation
      run: |
        ./path/to/flatten-rust/target/release/flatten-rust \
          --folders . \
          --skip-folders node_modules .git target \
          --output project_docs.md
          
    - name: Upload documentation
      uses: actions/upload-artifact@v2
      with:
        name: project-documentation
        path: project_docs.md
```

## Анализ результатов

### Структура выходного файла
```markdown
### DIRECTORY /path/to/project FOLDER STRUCTURE ###
📁 project/
    📁 src/
        📄 main.rs
        📄 lib.rs
    ⏭️ node_modules/ (skipped)
    📁 tests/
        📄 integration.rs
### DIRECTORY /path/to/project FOLDER STRUCTURE ###

### DIRECTORY /path/to/project FLATTENED CONTENT ###
### /path/to/project/src/main.rs BEGIN ###
fn main() {
    println!("Hello, world!");
}
### /path/to/project/src/main.rs END ###

### /path/to/project/src/lib.rs BEGIN ###
pub mod utils;
### /path/to/project/src/lib.rs END ###
### DIRECTORY /path/to/project FLATTENED CONTENT ###
```

### Поиск в выходном файле
```bash
# Найти все функции
grep -n "fn " codebase.md

# Найти все импорты
grep -n "use " codebase.md

# Найти все тесты
grep -n "#\[test\]" codebase.md

# Найти определённый файл
grep -A 10 "### /path/to/file.rs BEGIN ###" codebase.md
```

## Советы по оптимизации

### Для больших кодовых баз
1. **Используйте `--threads`** для параллельной обработки
2. **Ограничьте `--max-file-size`** для экономии памяти
3. **Пропускайте бинарные файлы** через `--skip-extensions`
4. **Используйте `--show-skipped`** для контроля пропускаемых папок

### Для регулярного использования
1. **Создайте alias** в shell:
   ```bash
   alias flatten='~/projects/flatten-rust/target/release/flatten-rust'
   ```

2. **Создайте скрипты** для типовых проектов:
   ```bash
   # flatten_rust.sh
   flatten --folders src tests docs --skip-folders target .git --output rust_project.md
   ```

3. **Используйте в Git hooks**:
   ```bash
   # .git/hooks/pre-commit
   ./flatten-rust --folders src --output pre_commit_check.md
   ```

## Решение проблем

### Недостаточно памяти
```bash
# Уменьшите количество потоков и размер файлов
./flatten-rust --folders . --threads 1 --max-file-size 10485760 --output low_mem.md
```

### Слишком медленно
```bash
# Увеличьте количество потоков
./flatten-rust --folders . --threads $(nproc) --output fast.md
```

### Слишком большой выходной файл
```bash
# Ограничьте размер анализируемых файлов
./flatten-rust --folders . --max-file-size 1048576 --output small.md
```

## Сравнение с Python версией

| Метрика | Python версия | Rust версия | Улучшение |
|---------|---------------|-------------|-----------|
| Скорость (5000 файлов) | 0.121s | 0.066s | 2x быстрее |
| Память | ~100MB | ~40MB | 2.5x меньше |
| Параллелизм | Нет | Да | ✅ |
| Memory mapping | Нет | Да | ✅ |
| Прогресс-бары | Нет | Да | ✅ |

Rust версия обеспечивает значительное преимущество в производительности и эффективности использования памяти, особенно на больших проектах.