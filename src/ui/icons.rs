//! Category icons. The database stores Lucide names; they are drawn straight
//! from the asset bundle.
use gpui_kit::{prelude::*, svg, Hsla, Pixels, Svg};

/// `(stored name, label)` in the order the picker shows them.
pub const CATEGORY_ICONS: &[(&str, &str)] = &[
    ("utensils", "Food"),
    ("car", "Car"),
    ("briefcase", "Work"),
    ("film", "Film"),
    ("shopping-bag", "Shopping"),
    ("zap", "Bills"),
    ("heart", "Health"),
    ("book-open", "Education"),
    ("coffee", "Coffee"),
    ("plane", "Travel"),
    ("home", "Home"),
    ("shopping-cart", "Cart"),
    ("wifi", "Internet"),
    ("smartphone", "Phone"),
    ("music", "Music"),
    ("gamepad-2", "Gaming"),
    ("gift", "Gift"),
    ("stethoscope", "Medical"),
    ("graduation-cap", "School"),
    ("building-2", "Office"),
    ("piggy-bank", "Savings"),
    ("credit-card", "Credit"),
    ("wallet", "Wallet"),
    ("wrench", "Repairs"),
    ("bot", "AI"),
    ("code", "Code"),
    ("laptop", "Laptop"),
    ("tv", "Streaming"),
    ("dumbbell", "Fitness"),
    ("dog", "Dog"),
    ("cat", "Cat"),
    ("leaf", "Nature"),
    ("shirt", "Clothing"),
    ("scissors", "Grooming"),
    ("bus", "Bus"),
    ("train", "Train"),
    ("hotel", "Hotel"),
    ("wine", "Drinks"),
    ("landmark", "Taxes"),
    ("shield", "Insurance"),
    ("cloud", "Cloud"),
    ("circle-question-mark", "Unaccounted"),
    ("user-round", "Woman"),
];

pub const DEFAULT_ICON: &str = "utensils";

/// Bundle file stem for a stored name. Two Lucide icons were renamed after
/// categories were first saved with the old names.
fn file_stem(name: &str) -> &str {
    match name {
        "home" => "house",
        "train" => "train-front",
        other => other,
    }
}

pub fn asset_path(name: &str) -> String {
    format!("icons/{}.svg", file_stem(name))
}

pub fn category_icon(name: &str, size: Pixels, color: Hsla) -> Svg {
    svg().path(asset_path(name)).size(size).flex_shrink_0().text_color(color)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::AssetSource as _;

    #[test]
    fn every_category_icon_is_in_the_bundle() {
        for (name, _) in CATEGORY_ICONS {
            let loaded = gpui_kit::assets::AllAssets.load(&asset_path(name)).unwrap();
            assert!(loaded.is_some(), "icon {name} missing from the asset bundle");
        }
    }

    #[test]
    fn renamed_icons_resolve() {
        assert_eq!(asset_path("home"), "icons/house.svg");
        assert_eq!(asset_path("train"), "icons/train-front.svg");
        assert_eq!(asset_path("utensils"), "icons/utensils.svg");
    }

    #[test]
    fn names_are_unique() {
        let mut names: Vec<_> = CATEGORY_ICONS.iter().map(|(n, _)| *n).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), CATEGORY_ICONS.len());
    }
}
