use crate::{
    GameType,
    config::{self, save_last_game},
    gacha_url,
};
use winsafe::{co::*, prelude::*, *};

const GAMES: [&str; 6] = [
    "原神",
    "Genshin Impact",
    "崩坏：星穹铁道",
    "Honkai: Star Rail",
    "绝区零",
    "Zenless Zone Zero",
];

#[derive(Clone)]
pub struct MainWindow {
    wnd: gui::WindowMain,
    games: gui::ComboBox,
    btn_get: gui::Button,
    btn_copy: gui::Button,
    url_preview: gui::Edit,
    sbar: gui::StatusBar,
}

impl MainWindow {
    pub fn new() -> Self {
        let wnd = gui::WindowMain::new(gui::WindowMainOpts {
            title: "获取抽卡链接".to_owned(),
            class_icon: gui::Icon::Id(101),
            size: (400, 265),
            style: gui::WindowMainOpts::default().style
                | co::WS::MINIMIZEBOX
                | co::WS::MAXIMIZEBOX
                | co::WS::SIZEBOX,
            ..Default::default()
        });

        let games = gui::ComboBox::new(
            &wnd,
            gui::ComboBoxOpts {
                position: (10, 8),
                width: 180,
                items: GAMES.iter().map(|&s| s.to_owned()).collect(),
                selected_item: config::load_last_game(),
                ..Default::default()
            },
        );

        let btn_get = gui::Button::new(
            &wnd,
            gui::ButtonOpts {
                text: "获取".to_owned(),
                position: (209, 5),
                resize_behavior: (gui::Horz::Repos, gui::Vert::None),
                ..Default::default()
            },
        );

        let btn_copy = gui::Button::new(
            &wnd,
            gui::ButtonOpts {
                text: "复制".to_owned(),
                position: (302, 5),
                resize_behavior: (gui::Horz::Repos, gui::Vert::None),
                ..Default::default()
            },
        );

        let url_preview = gui::Edit::new(
            &wnd,
            gui::EditOpts {
                position: (10, 40),
                width: 380,
                height: 200,
                edit_style: ES::MULTILINE
                    | ES::WANTRETURN
                    | ES::AUTOVSCROLL
                    | ES::NOHIDESEL
                    | ES::READONLY
                    | WS::VSCROLL.into(),
                resize_behavior: (gui::Horz::Resize, gui::Vert::Resize),
                ..Default::default()
            },
        );

        let sbar = gui::StatusBar::new(&wnd, &[gui::SbPart::Proportional(1)]);

        let new_self = Self {
            wnd,
            games,
            url_preview,
            btn_get,
            btn_copy,
            sbar,
        };
        new_self.events();
        new_self
    }

    pub fn run(&self) -> AnyResult<i32> {
        self.wnd.run_main(None)
    }

    fn set_status(&self, text: &str) {
        self.sbar.parts().set_texts(&[Some(format!("  {text}"))]);
    }

    fn get_url(&self) -> String {
        if let Some(s) = self.games.items().selected_text() {
            let game = match s.as_str() {
                "原神" => GameType::Hk4eCN,
                "Genshin Impact" => GameType::Hk4eGlobal,
                "崩坏：星穹铁道" => GameType::HkrpgCN,
                "Honkai: Star Rail" => GameType::HkrpgGlobal,
                "绝区零" => GameType::NapCN,
                "Zenless Zone Zero" => GameType::NapGlobal,
                _ => unreachable!("Invalid game selection"),
            };
            match gacha_url::get_gacha_url(game) {
                Ok(url) => {
                    self.set_status("获取链接成功");
                    return url.to_string();
                }
                Err(e) => self.set_status(&format!("获取链接失败：{}", e)),
            }
        } else {
            self.set_status("请选择游戏");
        }
        String::new()
    }

    fn set_clipboard(&self, text: &str) {
        if let Ok(_hclip_guard) = self.wnd.hwnd().OpenClipboard() {
            // 需要先清空剪贴板
            if !EmptyClipboard().is_ok() {
                self.set_status("无法清空剪贴板");
                return;
            }
            let c_string = match std::ffi::CString::new(text) {
                Ok(s) => s,
                Err(_) => {
                    self.set_status("链接包含无效字符");
                    return;
                }
            };
            // 将所有权转移给系统
            let ptr = Box::into_raw(Box::new(c_string)) as *mut u8;
            match unsafe { SetClipboardData(CF::TEXT, ptr) } {
                Ok(_) => self.set_status("已复制到剪贴板"),
                Err(e) => {
                    // 设置失败，手动释放内存
                    let _ = unsafe { Box::from_raw(ptr as *mut std::ffi::CString) };
                    self.set_status(&format!("无法复制到剪贴板：{}", e));
                }
            }
        } else {
            self.set_status("无法打开剪贴板");
        }
    }

    fn events(&self) {
        let self2 = self.clone();
        self.wnd.on().wm_create(move |_| {
            self2.url_preview.set_text(&self2.get_url());
            Ok(0)
        });

        let self2 = self.clone();
        self.games.on().cbn_sel_change(move || {
            if let Some(s) = self2.games.items().selected_text() {
                GAMES
                    .iter()
                    .position(|&g| g == s)
                    .map(|i| save_last_game(i.try_into().unwrap()));
            }
            self2.url_preview.set_text(&self2.get_url());
            Ok(())
        });

        let self2 = self.clone();
        self.btn_get.on().bn_clicked(move || {
            self2.url_preview.set_text(&self2.get_url());
            Ok(())
        });

        let self2 = self.clone();
        self.btn_copy.on().bn_clicked(move || {
            let text = self2.url_preview.text();
            if text.is_empty() {
                self2.set_status("没有链接可复制");
            } else {
                self2.set_clipboard(&text);
            }
            Ok(())
        });
    }
}
