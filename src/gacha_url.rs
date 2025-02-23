use crate::GameType;
use anyhow::{Context, Result};
use regex_lite::Regex;
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use url::Url;

pub fn check_url(url: Url) -> Option<Url> {
    let response = minreq::get(url.clone()).send().ok()?;
    let json: serde_json::Value = response.json().ok()?;
    if json["retcode"].as_i64() == Some(0) {
        Some(filter_url(url))
    } else {
        None
    }
}

pub fn filter_url(mut url: Url) -> Url {
    let query = url
        .query_pairs()
        .filter(|(key, _)| {
            matches!(
                key.as_ref(),
                "authkey" | "authkey_ver" | "sign_type" | "game_biz" | "lang"
            )
        })
        .fold(
            url::form_urlencoded::Serializer::new(String::new()),
            |mut serializer, (key, value)| {
                serializer.append_pair(&key, &value);
                serializer
            },
        )
        .finish();
    url.set_query(Some(&query));
    url
}

pub fn get_gacha_url(game_type: GameType) -> Result<Url> {
    let log_path = get_local_app_data_low_folder()?.join(match game_type {
        GameType::Hk4eCN => "miHoYo/原神/output_log.txt",
        GameType::Hk4eGlobal => "miHoYo/Genshin Impact/output_log.txt",
        GameType::HkrpgCN => "miHoYo/崩坏：星穹铁道/Player.log",
        GameType::HkrpgGlobal => "miHoYo/Cognosphere/Star Rail/Player.log",
        GameType::NapCN => "miHoYo/绝区零/Player.log",
        GameType::NapGlobal => "miHoYo/ZenlessZoneZero/Player.log",
    });

    let log_content =
        fs::read_to_string(&log_path).with_context(|| format!("未找到游戏日志 {:?}", log_path))?;
    get_gacha_url_with_log_data(log_content)
}

pub fn get_gacha_url_with_log_data(log_data: String) -> Result<Url> {
    let re = Regex::new(
        r"([A-Z]:/.*?(GenshinImpact_Data|YuanShen_Data|StarRail_Data|ZenlessZoneZero_Data))",
    )?;
    let captures = re.captures(&log_data).context("未找到游戏目录")?;
    get_gacha_url_with_game_data_path(&captures[0])
}

pub fn get_gacha_url_with_game_data_path(game_data_path: impl AsRef<Path>) -> Result<Url> {
    let re = Regex::new(r"(https://.+?/api/getGachaLog.+?authkey=.+?end_id=)")?;
    let web_cache_path = game_data_path.as_ref().join("webCaches");
    let cache_data = get_latest_folder(&web_cache_path)
        .with_context(|| format!("未找到游戏缓存目录 {:?}", web_cache_path))?
        .join("Cache/Cache_Data/data_2");

    String::from_utf8_lossy(
        &fs::read(&cache_data).with_context(|| format!("读取游戏缓存出错 {:?}", cache_data))?,
    )
    .split("1/0/")
    .collect::<Vec<_>>()
    .into_iter()
    .rev()
    .filter_map(|line| re.captures(line))
    .filter(|cap| cap.len() >= 1)
    .filter_map(|cap| Url::from_str(&cap[0]).ok())
    .find_map(check_url)
    .context("未找到链接")
}

fn get_local_app_data_low_folder() -> Result<PathBuf> {
    winsafe::SHGetKnownFolderPath(
        &winsafe::co::KNOWNFOLDERID::LocalAppDataLow,
        winsafe::co::KF::DEFAULT,
        None,
    )
    .map(PathBuf::from)
    .context("未找到 LocalLow 文件夹")
}

fn get_latest_folder(path: impl AsRef<Path>) -> Result<PathBuf> {
    fs::read_dir(&path)
        .with_context(|| format!("Failed to read directory {:?}", path.as_ref()))?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .max_by_key(|entry| entry.metadata().and_then(|meta| meta.modified()).ok())
        .map(|entry| entry.path())
        .context("未找到缓存目录")
}
