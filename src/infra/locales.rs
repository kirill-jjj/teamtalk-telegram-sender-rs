use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::{Loader, static_loader};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::LazyLock;
use unic_langid::LanguageIdentifier;

static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

static LANG_RU: LazyLock<LanguageIdentifier> =
    LazyLock::new(|| "ru".parse().expect("Valid RU langid"));
static LANG_EN: LazyLock<LanguageIdentifier> =
    LazyLock::new(|| "en".parse().expect("Valid EN langid"));

fn get_lang_id(lang_code: &str) -> &LanguageIdentifier {
    match lang_code {
        "ru" => &LANG_RU,
        _ => &LANG_EN,
    }
}

pub fn get_text(
    lang_code: &str,
    key: &str,
    args: Option<&HashMap<Cow<'static, str>, FluentValue>>,
) -> String {
    let lang_id = get_lang_id(lang_code);

    args.map_or_else(
        || LOCALES.lookup(lang_id, key),
        |args_map| LOCALES.lookup_with_args(lang_id, key, args_map),
    )
}

#[cfg(test)]
#[path = "../../tests/unit/infra_locales.rs"]
mod tests;

#[macro_export]
/// Helper to build Fluent arguments.
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
