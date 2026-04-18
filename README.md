# sosaltix2

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

**sosaltix2** — это маленькая операционная система, написанная на Rust, следуя ["Writing an OS in Rust"](https://os.phil-opp.com/). Проект реализует базовое ядро x86_64 с поддержкой прерываний, пейджинга, heap и асинхронного multitasking.

## Особенности

-  **Написано на Rust** — безопасность памяти без стандартной библиотеки
-  **Архитектура x86_64** — запускается на реальном железе или в QEMU
-  **Асинхронность** — поддержка async/await и кооперативная многозадачность
-  **VGA текстовый режим** — вывод на экран 80x25
-  **Ввод с клавиатуры** — обработка PS/2 клавиатуры
-  **Управление памятью** — пейджинг и heap allocation
-  **Встроенный шелл** — команды `echo`, `help`, `clear`

## Требования

- **Rust** (nightly) — проект использует nightly-фичи
- **QEMU** (опционально) — для эмуляции
- **bootimage** — для создания загрузочного образа

### Установка зависимостей

```bash
# Установка Rust (если ещё не установлен)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Переключение на nightly
rustup default nightly

# Установка компонентов
rustup component add rust-src
rustup component add llvm-tools-preview

# Установка bootimage
cargo install bootimage

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

# Создание загрузочного образа
cargo bootimage
```
### Запуск в QEMU 

```bash
# Запуск с автоматической сборкой
cargo run

# Или запуск готового образа
qemu-system-x86_64 -drive format=raw,file=target/x86_64-sosaltix2/debug/bootimage-sosaltix2.bin
```
### Сборка для реального пк

```bash
# Релизная сборка (меньший размер)
cargo bootimage --release

# Запись на USB-накопитель (замените /dev/sdX на ваше устройство!)
sudo dd if=target/x86_64-sosaltix2/release/bootimage-sosaltix2.bin of=/dev/sdX bs=4M status=progress
```
## Использование

Всего есть три команды: echo, clear, help. Сами поймёте крч
