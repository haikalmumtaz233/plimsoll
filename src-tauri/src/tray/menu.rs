use crate::domain::preferences::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Open,
    Quit,
}

impl MenuAction {
    const ALL: [Self; 2] = [Self::Open, Self::Quit];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Quit => "quit",
        }
    }

    pub const fn label(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::Open, Language::English) => "Open Plimsoll",
            (Self::Quit, Language::English) => "Quit",
            (Self::Open, Language::Indonesian) => "Buka Plimsoll",
            (Self::Quit, Language::Indonesian) => "Keluar",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::MenuAction;
    use crate::domain::preferences::Language;

    #[test]
    fn every_action_round_trips_through_its_id() {
        for action in MenuAction::ALL {
            assert_eq!(MenuAction::from_id(action.id()), Some(action));
        }
    }

    #[test]
    fn unknown_id_maps_to_no_action() {
        assert_eq!(MenuAction::from_id("settings"), None);
    }

    #[test]
    fn labels_follow_the_language() {
        assert_eq!(MenuAction::Open.label(Language::English), "Open Plimsoll");
        assert_eq!(MenuAction::Quit.label(Language::Indonesian), "Keluar");
        for language in Language::ALL {
            assert_ne!(
                MenuAction::Open.label(language),
                MenuAction::Quit.label(language)
            );
        }
    }

    #[test]
    fn ids_are_unique() {
        assert_ne!(MenuAction::Open.id(), MenuAction::Quit.id());
    }
}
