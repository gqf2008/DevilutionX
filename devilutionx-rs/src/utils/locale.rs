//! System locale helpers (C++ `Source/platform/locale.cpp`).
//!
//! Ports the pure `IetfToPosix` conversion; the platform-specific
//! `GetLocales()` probing (Windows/Android/Vita/3DS) is out of scope for the
//! desktop Rust build.

/// C++ `IetfToPosix(langCode)`: converts an IETF/BCP-47 language tag to a
/// POSIX locale id.
///
/// Simplified/Traditional Chinese map to `zh_CN` / `zh_TW` (the region does
/// not add value for those scripts); every other tag has its `-` separators
/// replaced with `_`.
pub fn ietf_to_posix(lang_code: &str) -> String {
    if lang_code.starts_with("zh-Hans") {
        return "zh_CN".to_string();
    }
    if lang_code.starts_with("zh-Hant") {
        return "zh_TW".to_string();
    }
    lang_code.replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_scripts_map_to_regions() {
        assert_eq!(ietf_to_posix("zh-Hans"), "zh_CN");
        assert_eq!(ietf_to_posix("zh-Hans-CN"), "zh_CN");
        assert_eq!(ietf_to_posix("zh-Hant"), "zh_TW");
        assert_eq!(ietf_to_posix("zh-Hant-TW"), "zh_TW");
    }

    #[test]
    fn test_dashes_become_underscores() {
        assert_eq!(ietf_to_posix("en-US"), "en_US");
        assert_eq!(ietf_to_posix("fr-FR"), "fr_FR");
        assert_eq!(ietf_to_posix("pt-BR"), "pt_BR");
    }

    #[test]
    fn test_already_posix_unchanged() {
        assert_eq!(ietf_to_posix("en_US"), "en_US");
        assert_eq!(ietf_to_posix("ja_JP"), "ja_JP");
    }
}
