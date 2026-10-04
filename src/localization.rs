use eframe::egui::{self, FontData, FontDefinitions, FontFamily};
use std::{env, fs, path::PathBuf};

pub const SYSTEM_LOCALE: &str = "system";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocaleOption {
    pub code: &'static str,
    pub native_name: &'static str,
}

pub const LOCALE_OPTIONS: [LocaleOption; 9] = [
    LocaleOption {
        code: SYSTEM_LOCALE,
        native_name: "System / 系统",
    },
    LocaleOption {
        code: "zh-CN",
        native_name: "中文",
    },
    LocaleOption {
        code: "en",
        native_name: "English",
    },
    LocaleOption {
        code: "ja",
        native_name: "日本語",
    },
    LocaleOption {
        code: "la",
        native_name: "Latina",
    },
    LocaleOption {
        code: "ko",
        native_name: "한국어",
    },
    LocaleOption {
        code: "ru",
        native_name: "Русский",
    },
    LocaleOption {
        code: "fr",
        native_name: "Français",
    },
    LocaleOption {
        code: "es",
        native_name: "Español",
    },
];

#[derive(Debug, Clone)]
pub struct LocaleManager {
    selection: &'static str,
    active: &'static str,
    system_locale: String,
}

impl LocaleManager {
    pub fn new(configured: &str) -> Self {
        let system_locale = sys_locale::get_locale().unwrap_or_else(|| "en".into());
        let selection = normalize_selection(configured);
        let active = if selection == SYSTEM_LOCALE {
            supported_locale_for(&system_locale)
        } else {
            selection
        };
        rust_i18n::set_locale(active);
        Self {
            selection,
            active,
            system_locale,
        }
    }

    pub fn selection(&self) -> &'static str {
        self.selection
    }

    pub fn system_locale(&self) -> &str {
        &self.system_locale
    }

    pub fn select(&mut self, selection: &str) {
        self.selection = normalize_selection(selection);
        self.active = if self.selection == SYSTEM_LOCALE {
            supported_locale_for(&self.system_locale)
        } else {
            self.selection
        };
        rust_i18n::set_locale(self.active);
    }

    pub fn active_native_name(&self) -> &'static str {
        LOCALE_OPTIONS
            .iter()
            .find(|option| option.code == self.active)
            .map_or("English", |option| option.native_name)
    }
}

fn normalize_selection(value: &str) -> &'static str {
    if value.eq_ignore_ascii_case(SYSTEM_LOCALE) {
        return SYSTEM_LOCALE;
    }
    let normalized = value.trim().replace('_', "-").to_ascii_lowercase();
    let language = normalized.split('-').next().unwrap_or_default();
    match language {
        "zh" => "zh-CN",
        "en" => "en",
        "ja" => "ja",
        "la" => "la",
        "ko" => "ko",
        "ru" => "ru",
        "fr" => "fr",
        "es" => "es",
        _ => SYSTEM_LOCALE,
    }
}

pub fn supported_locale_for(locale: &str) -> &'static str {
    let language = locale
        .trim()
        .replace('_', "-")
        .to_ascii_lowercase()
        .split('-')
        .next()
        .unwrap_or_default()
        .to_owned();
    match language.as_str() {
        "zh" => "zh-CN",
        "ja" => "ja",
        "la" => "la",
        "ko" => "ko",
        "ru" => "ru",
        "fr" => "fr",
        "es" => "es",
        _ => "en",
    }
}

/// Add common platform CJK fonts after egui's bundled fonts. Missing files are
/// ignored, so the same executable remains portable across desktop platforms.
pub fn install_system_fallback_fonts(ctx: &egui::Context) {
    let mut paths = Vec::<PathBuf>::new();
    if cfg!(windows) {
        let windows = env::var_os("WINDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
        let fonts = windows.join("Fonts");
        paths.extend([
            fonts.join("msyh.ttc"),
            fonts.join("YuGothM.ttc"),
            fonts.join("malgun.ttf"),
            fonts.join("arialuni.ttf"),
        ]);
    } else if cfg!(target_os = "macos") {
        paths.extend([
            PathBuf::from("/System/Library/Fonts/PingFang.ttc"),
            PathBuf::from("/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc"),
            PathBuf::from("/System/Library/Fonts/AppleSDGothicNeo.ttc"),
        ]);
    } else {
        paths.extend([
            PathBuf::from("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"),
            PathBuf::from("/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc"),
            PathBuf::from("/usr/share/fonts/opentype/noto/NotoSansCJKkr-Regular.otf"),
        ]);
    }

    let mut fonts = FontDefinitions::default();
    for (index, path) in paths.into_iter().enumerate() {
        let Ok(bytes) = fs::read(path) else {
            continue;
        };
        let name = format!("system-i18n-{index}");
        fonts
            .font_data
            .insert(name.clone(), FontData::from_owned(bytes).into());
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .push(name.clone());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .push(name);
    }
    ctx.set_fonts(fonts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_supported_system_locales_and_falls_back_to_english() {
        assert_eq!(supported_locale_for("zh_CN.UTF-8"), "zh-CN");
        assert_eq!(supported_locale_for("ja-JP"), "ja");
        assert_eq!(supported_locale_for("ko_KR"), "ko");
        assert_eq!(supported_locale_for("ru-RU"), "ru");
        assert_eq!(supported_locale_for("fr-CA"), "fr");
        assert_eq!(supported_locale_for("es-MX"), "es");
        assert_eq!(supported_locale_for("la"), "la");
        assert_eq!(supported_locale_for("de-DE"), "en");
    }

    #[test]
    fn invalid_saved_locale_returns_to_system_selection() {
        assert_eq!(normalize_selection("zh_CN"), "zh-CN");
        assert_eq!(normalize_selection("en-US"), "en");
        assert_eq!(normalize_selection("fr-CA"), "fr");
        assert_eq!(normalize_selection("unsupported"), SYSTEM_LOCALE);
    }

    #[test]
    fn every_supported_locale_has_its_own_primary_menu_translation() {
        for locale in ["en", "zh-CN", "ja", "la", "ko", "ru", "fr", "es"] {
            let file = rust_i18n::t!("menu.file", locale = locale);
            let language = rust_i18n::t!("menu.language", locale = locale);
            assert!(!file.is_empty(), "missing menu.file for {locale}");
            assert!(!language.is_empty(), "missing menu.language for {locale}");
        }
    }
}
