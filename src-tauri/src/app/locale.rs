use crate::domain::preferences::{Language, LanguageChoice};
use crate::i18n::Text;

#[must_use]
pub fn resolve(choice: LanguageChoice) -> Language {
    choice.resolve(sys_locale::get_locale().as_deref())
}

#[must_use]
pub fn text(choice: LanguageChoice) -> Text {
    Text::new(resolve(choice))
}
