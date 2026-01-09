use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{Loader, static_loader};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::OnceLock;
use unic_langid::LanguageIdentifier;

static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

static LANG_RU: OnceLock<LanguageIdentifier> = OnceLock::new();
static LANG_EN: OnceLock<LanguageIdentifier> = OnceLock::new();

fn get_lang_id(lang_code: &str) -> &'static LanguageIdentifier {
    match lang_code {
        "ru" => LANG_RU.get_or_init(|| "ru".parse().expect("Valid RU langid")),
        _ => LANG_EN.get_or_init(|| "en".parse().expect("Valid EN langid")),
    }
}

pub fn get_text(
    lang_code: &str,
    key: &str,
    args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
) -> String {
    let lang_id = get_lang_id(lang_code);

    if let Some(args_map) = args {
        LOCALES.lookup_with_args(lang_id, key, args_map)
    } else {
        LOCALES.lookup(lang_id, key)
    }
}

#[macro_export]
macro_rules! args {
    ( $($k:ident = $v:expr),* ) => {{
        let mut map = std::collections::HashMap::new();
        $(
            map.insert(
                std::borrow::Cow::Borrowed(stringify!($k)),
                fluent_templates::fluent_bundle::FluentValue::from($v)
            );
        )*
        Some(map)
    }};
}
