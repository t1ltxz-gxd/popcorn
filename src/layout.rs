use once_cell::sync::Lazy;
use std::collections::HashMap;

/// EN → RU keys match by physical position on the keyboard
/// (standard ЙЦУКЕН/QWERTY Windows layout).
const PAIRS: &[(char, char)] = &[
    ('`', 'ё'),
    ('q', 'й'), ('w', 'ц'), ('e', 'у'), ('r', 'к'), ('t', 'е'),
    ('y', 'н'), ('u', 'г'), ('i', 'ш'), ('o', 'щ'), ('p', 'з'),
    ('[', 'х'), (']', 'ъ'),
    ('a', 'ф'), ('s', 'ы'), ('d', 'в'), ('f', 'а'), ('g', 'п'),
    ('h', 'р'), ('j', 'о'), ('k', 'л'), ('l', 'д'), (';', 'ж'), ('\'', 'э'),
    ('z', 'я'), ('x', 'ч'), ('c', 'с'), ('v', 'м'), ('b', 'и'),
    ('n', 'т'), ('m', 'ь'), (',', 'б'), ('.', 'ю'), ('/', '.'),
];

/// An EN character (including an uppercase character) → a RU character of the same physical key.
static EN_TO_RU: Lazy<HashMap<char, char>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for &(en, ru) in PAIRS {
        map.insert(en, ru);
        // заглавные буквы — та же позиция клавиши + Shift
        for (en_upper, ru_upper) in en.to_uppercase().zip(ru.to_uppercase()) {
            map.insert(en_upper, ru_upper);
        }
    }
    map
});

/// Translates the line as if it were printed in Russian
/// layout with the same physical keys that gave EN characters.
/// Symbols that are not in the table (numbers, already in Cyrillic, etc.),
/// remain unchanged.
pub fn translate_en_to_ru(input: &str) -> String {
    input
        .chars()
        .map(|c| EN_TO_RU.get(&c).copied().unwrap_or(c))
        .collect()
}
