//! Locale, direction, and chrome string catalogs.
//!
//! Chrome that mirrors uses start/end ([`order`], [`align_start`],
//! [`align_end`], [`inline_pad`]). iced `Alignment::Start` is physical
//! left. Paths, URLs, and code stay left-to-right islands. Arabic,
//! Urdu, and Persian clocks use Eastern digits; Hebrew uses 123.

use std::collections::BTreeMap;

/// Text direction.
///
/// ```
/// use icedtea::i18n::{direction_for, Direction};
/// assert_eq!(direction_for("ar"), Direction::Rtl);
/// assert_eq!(direction_for("en"), Direction::Ltr);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Ltr,
    Rtl,
}

/// Clock face digits. Arabic, Urdu, and Persian use Eastern; Hebrew uses 123.
///
/// ```
/// use icedtea::i18n::ClockDigits;
/// assert_eq!(ClockDigits::for_lang("ar"), ClockDigits::Eastern);
/// assert_eq!(ClockDigits::for_lang("he"), ClockDigits::Western);
/// assert_eq!(ClockDigits::western_str("٩٠%"), "90%");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockDigits {
    Western,
    Eastern,
}

impl ClockDigits {
    const EASTERN: [char; 10] = ['٠', '١', '٢', '٣', '٤', '٥', '٦', '٧', '٨', '٩'];

    /// Digit set for a BCP 47 primary language.
    pub fn for_lang(lang: &str) -> Self {
        let primary = lang
            .split(['-', '_'])
            .next()
            .unwrap_or("en")
            .to_ascii_lowercase();
        match primary.as_str() {
            "ar" | "fa" | "ur" => Self::Eastern,
            _ => Self::Western,
        }
    }

    /// Map ASCII 0-9 in `s` onto this set. `%` becomes the Arabic
    /// percent sign so `40%` stays one run (`٤٠٪`) instead of a
    /// bidi-split `٪٤`. Other characters stay.
    pub fn map_str(self, s: &str) -> String {
        if self != Self::Eastern {
            return s.to_string();
        }
        s.chars()
            .map(|c| {
                if let Some(d) = c.to_digit(10) {
                    Self::EASTERN[d as usize]
                } else if c == '%' {
                    '٪'
                } else {
                    c
                }
            })
            .collect()
    }

    /// Map Eastern digits and the Arabic percent sign in `s` back to
    /// ASCII. Other characters stay.
    pub fn western_str(s: &str) -> String {
        s.chars()
            .map(|c| {
                if c == '٪' {
                    return '%';
                }
                Self::EASTERN
                    .iter()
                    .position(|&e| e == c)
                    .and_then(|d| char::from_digit(d as u32, 10))
                    .unwrap_or(c)
            })
            .collect()
    }
}

/// Application locale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locale {
    pub lang: String,
    pub direction: Direction,
}

impl Locale {
    pub const ENGLISH: Locale = Locale {
        lang: String::new(),
        direction: Direction::Ltr,
    };

    pub fn new(lang: impl Into<String>) -> Self {
        let lang = lang.into();
        let direction = direction_for(&lang);
        Self { lang, direction }
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::new("en")
    }
}

/// Reverse a sequence when direction is RTL.
///
/// ```
/// use icedtea::i18n::{order, Direction};
/// assert_eq!(order(Direction::Rtl, [1, 2, 3]), vec![3, 2, 1]);
/// assert_eq!(order(Direction::Ltr, [1, 2, 3]), vec![1, 2, 3]);
/// ```
pub fn order<T>(dir: Direction, items: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut v: Vec<T> = items.into_iter().collect();
    if dir == Direction::Rtl {
        v.reverse();
    }
    v
}

/// Cross-axis start: left in LTR, right in RTL.
///
/// iced `Alignment::Start` is physical left. Use this for start-edge
/// labels, list titles, and other inline chrome.
///
/// ```
/// use icedtea::i18n::{align_start, Direction};
/// assert_eq!(align_start(Direction::Ltr), icedtea::iced::Alignment::Start);
/// assert_eq!(align_start(Direction::Rtl), icedtea::iced::Alignment::End);
/// ```
pub fn align_start(dir: Direction) -> iced::Alignment {
    match dir {
        Direction::Ltr => iced::Alignment::Start,
        Direction::Rtl => iced::Alignment::End,
    }
}

/// Cross-axis end: right in LTR, left in RTL.
pub fn align_end(dir: Direction) -> iced::Alignment {
    match dir {
        Direction::Ltr => iced::Alignment::End,
        Direction::Rtl => iced::Alignment::Start,
    }
}

/// Horizontal start for iced `text_input` placeholder and value.
///
/// iced `align_x` takes physical Left/Right. This is the start edge.
///
/// ```
/// use icedtea::i18n::{align_x_start, Direction};
/// assert_eq!(align_x_start(Direction::Ltr), iced::alignment::Horizontal::Left);
/// assert_eq!(align_x_start(Direction::Rtl), iced::alignment::Horizontal::Right);
/// ```
pub fn align_x_start(dir: Direction) -> iced::alignment::Horizontal {
    match dir {
        Direction::Ltr => iced::alignment::Horizontal::Left,
        Direction::Rtl => iced::alignment::Horizontal::Right,
    }
}

/// Physical left/right padding from start/end amounts.
///
/// ```
/// use icedtea::i18n::{inline_pad, Direction};
/// assert_eq!(inline_pad(Direction::Ltr, 8.0, 12.0), (8.0, 12.0));
/// assert_eq!(inline_pad(Direction::Rtl, 8.0, 12.0), (12.0, 8.0));
/// ```
pub fn inline_pad(dir: Direction, start: f32, end: f32) -> (f32, f32) {
    match dir {
        Direction::Ltr => (start, end),
        Direction::Rtl => (end, start),
    }
}

/// BCP 47 primary language → direction.
pub fn direction_for(lang: &str) -> Direction {
    let primary = lang
        .split(['-', '_'])
        .next()
        .unwrap_or("en")
        .to_ascii_lowercase();
    match primary.as_str() {
        "ar" | "fa" | "he" | "ur" => Direction::Rtl,
        _ => Direction::Ltr,
    }
}

/// Translate key → string. Missing keys return the key.
///
/// ```
/// let mut cat = icedtea::i18n::Catalog::builtin();
/// assert_eq!(cat.t("ok"), "OK");
/// cat.insert("ok", "D'accord");
/// assert_eq!(cat.t("ok"), "D'accord");
/// ```
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    map: BTreeMap<String, String>,
}

impl Catalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builtin() -> Self {
        Self::for_locale(&Locale::default())
    }

    /// Chrome catalog for a locale; records direction for `run` / chrome.
    ///
    /// Built-in tables: English (`en`), Vietnamese (`vi`), Japanese
    /// (`ja`), Chinese (`zh`), Arabic (`ar`), and Urdu (`ur`).
    /// Unknown primaries use English. Arabic and Urdu set `direction`
    /// to `rtl`.
    ///
    /// ```
    /// use icedtea::i18n::{direction_for, Catalog, Locale};
    /// let vi = Catalog::for_locale(&Locale::new("vi"));
    /// assert_eq!(vi.t("save"), "Lưu");
    /// assert_eq!(direction_for("vi"), icedtea::i18n::Direction::Ltr);
    /// let ja = Catalog::for_locale(&Locale::new("ja"));
    /// assert_eq!(ja.t("file"), "ファイル");
    /// ```
    pub fn for_locale(locale: &Locale) -> Self {
        let mut c = Self::new();
        c.insert("lang", locale.lang.clone());
        c.insert(
            "direction",
            match locale.direction {
                Direction::Rtl => "rtl",
                Direction::Ltr => "ltr",
            },
        );
        for (k, v) in chrome_strings(&locale.lang) {
            c.insert(*k, *v);
        }
        c
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.map.insert(key.into(), value.into());
    }

    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.map.get(key).map(String::as_str).unwrap_or(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.map.keys().map(String::as_str)
    }
}

fn primary_lang(lang: &str) -> &str {
    lang.split(['-', '_']).next().unwrap_or("en")
}

fn chrome_strings(lang: &str) -> &'static [(&'static str, &'static str)] {
    match primary_lang(lang).to_ascii_lowercase().as_str() {
        "vi" => &[
            ("ok", "OK"),
            ("cancel", "Hủy"),
            ("close", "Đóng"),
            ("save", "Lưu"),
            ("open", "Mở"),
            ("new", "Mới"),
            ("copy", "Sao chép"),
            ("select-all", "Chọn tất cả"),
            ("delete", "Xóa"),
            ("preferences", "Tùy chọn"),
            ("about", "Giới thiệu"),
            ("search", "Tìm kiếm"),
            ("command-palette", "Bảng lệnh"),
            ("back", "Quay lại"),
            ("undo", "Hoàn tác"),
            ("redo", "Làm lại"),
            ("file", "Tệp"),
            ("edit", "Chỉnh sửa"),
            ("view", "Xem"),
            ("help", "Trợ giúp"),
            ("go", "Đi"),
            ("theme", "Giao diện"),
            ("density", "Mật độ"),
            ("empty", "Chưa có gì"),
        ],
        "ja" => &[
            ("ok", "OK"),
            ("cancel", "キャンセル"),
            ("close", "閉じる"),
            ("save", "保存"),
            ("open", "開く"),
            ("new", "新規"),
            ("copy", "コピー"),
            ("select-all", "すべて選択"),
            ("delete", "削除"),
            ("preferences", "設定"),
            ("about", "情報"),
            ("search", "検索"),
            ("command-palette", "コマンドパレット"),
            ("back", "戻る"),
            ("undo", "元に戻す"),
            ("redo", "やり直し"),
            ("file", "ファイル"),
            ("edit", "編集"),
            ("view", "表示"),
            ("help", "ヘルプ"),
            ("go", "移動"),
            ("theme", "テーマ"),
            ("density", "密度"),
            ("empty", "まだありません"),
        ],
        "zh" => &[
            ("ok", "确定"),
            ("cancel", "取消"),
            ("close", "关闭"),
            ("save", "保存"),
            ("open", "打开"),
            ("new", "新建"),
            ("copy", "复制"),
            ("select-all", "全选"),
            ("delete", "删除"),
            ("preferences", "偏好设置"),
            ("about", "关于"),
            ("search", "搜索"),
            ("command-palette", "命令面板"),
            ("back", "返回"),
            ("undo", "撤销"),
            ("redo", "重做"),
            ("file", "文件"),
            ("edit", "编辑"),
            ("view", "查看"),
            ("help", "帮助"),
            ("go", "转到"),
            ("theme", "主题"),
            ("density", "密度"),
            ("empty", "暂无内容"),
        ],
        "he" => &[
            ("ok", "אישור"),
            ("cancel", "ביטול"),
            ("close", "סגירה"),
            ("save", "שמירה"),
            ("open", "פתיחה"),
            ("new", "חדש"),
            ("copy", "העתקה"),
            ("select-all", "בחירת הכל"),
            ("delete", "מחיקה"),
            ("preferences", "העדפות"),
            ("about", "אודות"),
            ("search", "חיפוש"),
            ("command-palette", "לוח פקודות"),
            ("back", "חזרה"),
            ("undo", "ביטול פעולה"),
            ("redo", "ביצוע חוזר"),
            ("file", "קובץ"),
            ("edit", "עריכה"),
            ("view", "תצוגה"),
            ("help", "עזרה"),
            ("go", "מעבר"),
            ("theme", "ערכת נושא"),
            ("density", "צפיפות"),
            ("empty", "עדיין אין כאן כלום"),
        ],
        "ar" => &[
            ("ok", "حسناً"),
            ("cancel", "إلغاء"),
            ("close", "إغلاق"),
            ("save", "حفظ"),
            ("open", "فتح"),
            ("new", "جديد"),
            ("copy", "نسخ"),
            ("select-all", "تحديد الكل"),
            ("delete", "حذف"),
            ("preferences", "تفضيلات"),
            ("about", "حول"),
            ("search", "بحث"),
            ("command-palette", "لوحة الأوامر"),
            ("back", "رجوع"),
            ("undo", "تراجع"),
            ("redo", "إعادة"),
            ("file", "ملف"),
            ("edit", "تحرير"),
            ("view", "عرض"),
            ("help", "مساعدة"),
            ("go", "انتقال"),
            ("theme", "سمة"),
            ("density", "الكثافة"),
            ("empty", "لا شيء بعد"),
        ],
        "ur" => &[
            ("ok", "ٹھیک ہے"),
            ("cancel", "منسوخ"),
            ("close", "بند کریں"),
            ("save", "محفوظ کریں"),
            ("open", "کھولیں"),
            ("new", "نیا"),
            ("copy", "نقل"),
            ("select-all", "سب منتخب کریں"),
            ("delete", "حذف"),
            ("preferences", "ترجیحات"),
            ("about", "تعارف"),
            ("search", "تلاش"),
            ("command-palette", "کمانڈ پیلیٹ"),
            ("back", "واپس"),
            ("undo", "کالعدم"),
            ("redo", "دہرائیں"),
            ("file", "فائل"),
            ("edit", "ترمیم"),
            ("view", "منظر"),
            ("help", "مدد"),
            ("go", "جائیں"),
            ("theme", "تھیم"),
            ("density", "کثافت"),
            ("empty", "ابھی کچھ نہیں"),
        ],
        _ => &[
            ("ok", "OK"),
            ("cancel", "Cancel"),
            ("close", "Close"),
            ("save", "Save"),
            ("open", "Open"),
            ("new", "New"),
            ("copy", "Copy"),
            ("select-all", "Select all"),
            ("delete", "Delete"),
            ("preferences", "Preferences"),
            ("about", "About"),
            ("search", "Search"),
            ("command-palette", "Command palette"),
            ("back", "Back"),
            ("undo", "Undo"),
            ("redo", "Redo"),
            ("file", "File"),
            ("edit", "Edit"),
            ("view", "View"),
            ("help", "Help"),
            ("go", "Go"),
            ("theme", "Theme"),
            ("density", "Density"),
            ("empty", "Nothing here yet"),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_direction_and_catalog() {
        assert_eq!(direction_for("en-US"), Direction::Ltr);
        assert_eq!(direction_for("he"), Direction::Rtl);
        assert_eq!(ClockDigits::for_lang("he"), ClockDigits::Western);
        assert_eq!(ClockDigits::for_lang("ar"), ClockDigits::Eastern);
        assert_eq!(ClockDigits::for_lang("fa-IR"), ClockDigits::Eastern);
        assert_eq!(ClockDigits::for_lang("ur"), ClockDigits::Eastern);
        assert_eq!(ClockDigits::for_lang("en"), ClockDigits::Western);
        assert_eq!(ClockDigits::Eastern.map_str("40% · 1 min"), "٤٠٪ · ١ min");
        assert_eq!(ClockDigits::Western.map_str("40%"), "40%");
        assert_eq!(ClockDigits::western_str("٩٠%"), "90%");
        assert_eq!(ClockDigits::western_str("٩٠٪"), "90%");
        assert_eq!(ClockDigits::western_str("100%"), "100%");
        assert_eq!(direction_for("fa-IR"), Direction::Rtl);
        assert_eq!(direction_for("ur"), Direction::Rtl);
        assert_eq!(direction_for(""), Direction::Ltr);
        let loc = Locale::new("ar");
        assert_eq!(loc.direction, Direction::Rtl);
        assert_eq!(Locale::default().lang, "en");
        let _ = Locale::ENGLISH;
        let ar = Catalog::for_locale(&loc);
        assert_eq!(ar.t("direction"), "rtl");
        assert_eq!(ar.t("lang"), "ar");
        let mut cat = Catalog::builtin();
        assert_eq!(cat.t("ok"), "OK");
        assert_eq!(cat.t("direction"), "ltr");
        assert_eq!(cat.t("missing-key"), "missing-key");
        cat.insert("ok", "Oui");
        assert_eq!(cat.t("ok"), "Oui");
        assert!(cat.keys().any(|k| k == "cancel"));
        assert!(Catalog::new().t("x") == "x");
        assert_eq!(order(Direction::Rtl, ["a", "b"]), vec!["b", "a"]);
        assert_eq!(order(Direction::Ltr, ["a", "b"]), vec!["a", "b"]);
        assert_eq!(align_start(Direction::Ltr), iced::Alignment::Start);
        assert_eq!(align_start(Direction::Rtl), iced::Alignment::End);
        assert_eq!(align_end(Direction::Ltr), iced::Alignment::End);
        assert_eq!(align_end(Direction::Rtl), iced::Alignment::Start);
        assert_eq!(
            align_x_start(Direction::Ltr),
            iced::alignment::Horizontal::Left
        );
        assert_eq!(
            align_x_start(Direction::Rtl),
            iced::alignment::Horizontal::Right
        );
        assert_eq!(inline_pad(Direction::Ltr, 8.0, 12.0), (8.0, 12.0));
        assert_eq!(inline_pad(Direction::Rtl, 8.0, 12.0), (12.0, 8.0));
    }

    #[test]
    fn catalog_scripts_match_locale_and_stay_ltr() {
        for (tag, save, file, search) in [
            ("en", "Save", "File", "Search"),
            ("vi", "Lưu", "Tệp", "Tìm kiếm"),
            ("vi-VN", "Lưu", "Tệp", "Tìm kiếm"),
            ("ja", "保存", "ファイル", "検索"),
            ("zh", "保存", "文件", "搜索"),
            ("zh-CN", "保存", "文件", "搜索"),
        ] {
            assert_eq!(direction_for(tag), Direction::Ltr, "{tag}");
            let loc = Locale::new(tag);
            assert_eq!(loc.direction, Direction::Ltr);
            let cat = Catalog::for_locale(&loc);
            assert_eq!(cat.t("save"), save, "{tag}");
            assert_eq!(cat.t("file"), file, "{tag}");
            assert_eq!(cat.t("search"), search, "{tag}");
            assert_eq!(cat.t("direction"), "ltr");
        }
        let unknown = Catalog::for_locale(&Locale::new("sv"));
        assert_eq!(unknown.t("ok"), "OK");
        let empty = Catalog::for_locale(&Locale::ENGLISH);
        assert_eq!(empty.t("save"), "Save");
        assert_eq!(chrome_strings("").first().map(|p| p.0), Some("ok"));
        let loc = Locale::new("ja");
        let ja = Catalog::for_locale(&loc);
        let mut table = crate::action::ActionTable::new();
        table.insert(crate::action::Action::new("file.save", ja.t("save"), ()));
        let tok = crate::theme::named("dark").tokens;
        let _: crate::Element<'_, ()> = crate::pattern::menu_bar(&table, tok, loc.direction, &ja);
        let _: crate::Element<'_, ()> = crate::pattern::toolbar(table.iter(), tok, Direction::Ltr);
        let _: crate::Element<'_, ()> =
            crate::pattern::status_bar("ok", None, None, &table, tok, Direction::Ltr);
    }

    #[test]
    fn catalog_scripts_match_rtl_locales() {
        for (tag, save, file, search) in [
            ("ar", "حفظ", "ملف", "بحث"),
            ("ar-EG", "حفظ", "ملف", "بحث"),
            ("ur", "محفوظ کریں", "فائل", "تلاش"),
            ("ur-PK", "محفوظ کریں", "فائل", "تلاش"),
            ("he", "שמירה", "קובץ", "חיפוש"),
            ("he-IL", "שמירה", "קובץ", "חיפוש"),
        ] {
            assert_eq!(direction_for(tag), Direction::Rtl, "{tag}");
            let loc = Locale::new(tag);
            assert_eq!(loc.direction, Direction::Rtl);
            let cat = Catalog::for_locale(&loc);
            assert_eq!(cat.t("save"), save, "{tag}");
            assert_eq!(cat.t("file"), file, "{tag}");
            assert_eq!(cat.t("search"), search, "{tag}");
            assert_eq!(cat.t("direction"), "rtl");
        }
        let loc = Locale::new("ar");
        let ar = Catalog::for_locale(&loc);
        let mut table = crate::action::ActionTable::new();
        table.insert(crate::action::Action::new("file.save", ar.t("save"), ()));
        let tok = crate::theme::named("dark").tokens;
        let _: crate::Element<'_, ()> = crate::pattern::menu_bar(&table, tok, loc.direction, &ar);
        let _: crate::Element<'_, ()> = crate::pattern::toolbar(table.iter(), tok, loc.direction);
        let _: crate::Element<'_, ()> =
            crate::pattern::status_bar("ok", None, None, &table, tok, loc.direction);
    }
}
