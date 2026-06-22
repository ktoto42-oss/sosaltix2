# sosaltix2

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange)](https://www.rust-lang.org/)

**sosaltix2** — это маленькая операционная система, написанная на Rust, частично следуя ["Writing an OS in Rust"](https://os.phil-opp.com/). Проект реализует базовое ядро x86_64 с поддержкой прерываний, пейджинга, heap и асинхронной кооперативной многозадачности.

## Особенности

-  **Написано на Rust** - безопасность памяти без стандартной библиотеки
-  **Архитектура x86_64** - запускается в QEMU
-  **Асинхронность** - поддержка async/await и кооперативная многозадачность
-  **Ввод с клавиатуры** - обработка PS/2 клавиатуры
-  **Управление памятью** - пейджинг и heap allocation
-  **Поддержка FAT32** - умеет создавать, удалять, изменять и читать файлы

## Требования

- **Rust** (nightly) — проект использует nightly-фичи
- **QEMU** (опционально) — для эмуляции

### Установка зависимостей

```bash
# Установка Rust (если ещё не установлен)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Или через репозитории (Arch linux) 
sudo pacman -S rustup

# Переключение на nightly
rustup default nightly

# Установка компонентов
rustup component add rust-src
rustup component add llvm-tools-preview

# Установка QEMU (Arch Linux)
sudo pacman -S qemu

```

### Сборка

```bash
# Клонирование репозитория
git clone https://github.com/ktoto42-oss/sosaltix2.git
cd sosaltix2

# Сборка проекта
cargo build

# Запуск (UEFI)
cargo run uefi

```
