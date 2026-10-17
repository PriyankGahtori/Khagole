//! Chinese astrology: the calendar of a moment ([`calendar`]), and the Four Pillars
//! ([`bazi`]) and the Purple Star chart ([`ziwei`]) cast from it.
//!
//! Not part of the reference implementation. Results carry stable codes only (pinyin
//! for stems, branches and solar terms, English for elements, animals, Ten Gods, NaYin
//! and life stages); wording belongs to the application.

pub(crate) mod bazi;
pub(crate) mod calendar;
pub(crate) mod jie_table;
pub(crate) mod lunar_table;
pub(crate) mod ziwei;

/// The ten Heavenly Stems, 甲 to 癸.
pub(crate) const STEMS: [&str; 10] = [
    "jia", "yi", "bing", "ding", "wu", "ji", "geng", "xin", "ren", "gui",
];

/// The twelve Earthly Branches, 子 to 亥.
pub(crate) const BRANCHES: [&str; 12] = [
    "zi", "chou", "yin", "mao", "chen", "si", "wu", "wei", "shen", "you", "xu", "hai",
];

/// The animal of each branch.
pub(crate) const ANIMALS: [&str; 12] = [
    "rat", "ox", "tiger", "rabbit", "dragon", "snake", "horse", "goat", "monkey", "rooster", "dog",
    "pig",
];

/// The five phases in their generating order: each produces the next and controls the one
/// after that.
pub(crate) const ELEMENTS: [&str; 5] = ["wood", "fire", "earth", "metal", "water"];

/// Element index (into [`ELEMENTS`]) of each branch.
const BRANCH_ELEMENTS: [usize; 12] = [4, 2, 0, 0, 2, 1, 1, 2, 3, 3, 2, 4];

/// Element index of a stem: two stems, yang then yin, per element.
pub(crate) fn stem_element(stem: usize) -> usize {
    stem / 2
}

/// Element index of a branch.
pub(crate) fn branch_element(branch: usize) -> usize {
    BRANCH_ELEMENTS[branch]
}

/// "yang" for even stems and branches, "yin" for odd ones.
pub(crate) fn polarity(index: usize) -> &'static str {
    if index % 2 == 0 {
        "yang"
    } else {
        "yin"
    }
}

/// Position (0-59) in the sexagenary cycle of a stem and a branch of the same polarity.
pub(crate) fn cycle_index(stem: usize, branch: usize) -> usize {
    (6 * stem as i64 - 5 * branch as i64).rem_euclid(60) as usize
}

/// Position in the sexagenary cycle of a Gregorian year (1984 is 甲子).
pub(crate) fn year_cycle_index(year: i32) -> usize {
    (i64::from(year) - 4).rem_euclid(60) as usize
}
