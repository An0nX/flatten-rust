//! # Flatten Rust (Binary Crate)
//!
//! Этот крейт предоставляет исполняемый файл для утилиты `flatten-rust`.
//! Он служит тонкой оберткой вокруг библиотеки `flatten_rust`, отвечая за
//! парсинг аргументов командной строки и запуск основного процесса.

use anyhow::Result;
use clap::Parser;
use flatten_rust::Args;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    flatten_rust::run(&args).await
}
