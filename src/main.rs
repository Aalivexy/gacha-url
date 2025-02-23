#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod config;
mod gacha_url;
mod ui;

use winsafe::{prelude::*, *};

#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub enum GameType {
    Hk4eCN,
    Hk4eGlobal,
    HkrpgCN,
    HkrpgGlobal,
    NapCN,
    NapGlobal,
}

fn main() {
    if let Err(e) = ui::MainWindow::new().run() {
        HWND::NULL
            .MessageBox(&e.to_string(), "Error", co::MB::ICONERROR)
            .unwrap();
    }
}
