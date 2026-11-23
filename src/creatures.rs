/// Classification of marine creature types in the aquarium ecosystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreatureKind {
    SmallFish,
    MediumFish,
    LargeFish,
    Jellyfish,
    Crab,
}

/// Multi-line directional ASCII sprite with distinct right-facing and left-facing art.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectionalSprite {
    pub name: &'static str,
    pub right: &'static [&'static str],
    pub left: &'static [&'static str],
}

/// Multi-line animated ASCII sprite cycling through 2 or more frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimatedSprite {
    pub name: &'static str,
    pub frames: &'static [&'static [&'static str]],
}

/// Catalog of distinct real medium-sized fish species (3 rows tall).
pub const MEDIUM_FISH_SPECIES: &[DirectionalSprite] = &[
    // Tropical Angelfish / Snapper (width 9, height 3)
    DirectionalSprite {
        name: "angelfish",
        right: &[
            "   _/\\   ",
            "><(((  o>",
            "   \\_/   ",
        ],
        left: &[
            "   /\\_   ",
            "<o  )))><",
            "   \\_/   ",
        ],
    },
    // Spiny Pufferfish (width 8, height 3)
    DirectionalSprite {
        name: "pufferfish",
        right: &[
            "  \\^|^/ ",
            "><(  o)>",
            "  /_|_\\ ",
        ],
        left: &[
            " \\^|^/  ",
            "<(o  )><",
            " /_|_\\  ",
        ],
    },
    // Striped Clownfish (width 10, height 3)
    DirectionalSprite {
        name: "clownfish",
        right: &[
            "   .--.   ",
            "><((| | o>",
            "   '--'   ",
        ],
        left: &[
            "   .--.   ",
            "<o | |))><",
            "   '--'   ",
        ],
    },
];

/// Catalog of iconic large ocean species (5 rows tall).
pub const LARGE_FISH_SPECIES: &[DirectionalSprite] = &[
    // Shark (Hiu) — width 20, height 5
    DirectionalSprite {
        name: "shark",
        right: &[
            "         /\\         ",
            "\\      _/  \\______  ",
            " \\____/     ///  o\\ ",
            " /    \\      _____/ ",
            "/      \\____/       ",
        ],
        left: &[
            "         /\\         ",
            "  ______/  \\_      /",
            " /o  \\\\\\     \\____/ ",
            " \\_____      /    \\ ",
            "       \\____/      \\",
        ],
    },
    // Swordfish (Ikan Pedang) — width 24, height 5
    DirectionalSprite {
        name: "swordfish",
        right: &[
            "        /|\\             ",
            "\\     _/ | \\___         ",
            " \\___/      o  \\=======>",
            " /   \\_________/        ",
            "/       \\_/             ",
        ],
        left: &[
            "             /|\\        ",
            "         ___/ | \\_     /",
            "<=======/  o      \\___/ ",
            "        \\_________/   \\ ",
            "             \\_/       \\",
        ],
    },
    // Whale (Paus) — width 21, height 5
    DirectionalSprite {
        name: "whale",
        right: &[
            "      .---------.    ",
            "\\   _/           \\   ",
            " \\_/           o  \\  ",
            " / \\______________/  ",
            "/       \\___/        ",
        ],
        left: &[
            "    .---------.      ",
            "   /           \\_   /",
            "  /  o           \\_/ ",
            "  \\______________/ \\ ",
            "        \\___/       \\",
        ],
    },
];

/// Catalog of animated pulsing jellyfish species (4 rows tall, 2 frames).
pub const JELLYFISH_SPECIES: &[AnimatedSprite] = &[AnimatedSprite {
    name: "jellyfish",
    frames: &[
        &[
            " .---. ",
            "(_o_o_)",
            " / | \\ ",
            " \\ | / ",
        ],
        &[
            " ,---, ",
            "(-o-o-)",
            " \\ | / ",
            " / | \\ ",
        ],
    ],
}];

/// Catalog of animated seabed crab species (2 rows tall, 2 frames).
pub const CRAB_SPECIES: &[AnimatedSprite] = &[AnimatedSprite {
    name: "crab",
    frames: &[
        &[
            " V\\_o_o_/V ",
            " /=(___)=\\ ",
        ],
        &[
            " v\\_o_o_/v ",
            " \\=(___)=/ ",
        ],
    ],
}];

#[cfg(test)]
mod tests {
    use super::*;

    fn mirror_ascii_char(ch: char) -> char {
        match ch {
            '/' => '\\',
            '\\' => '/',
            '<' => '>',
            '>' => '<',
            '(' => ')',
            ')' => '(',
            '[' => ']',
            ']' => '[',
            '{' => '}',
            '}' => '{',
            other => other,
        }
    }

    fn mirror_ascii_line(line: &str) -> String {
        line.chars().rev().map(mirror_ascii_char).collect()
    }

    fn assert_directional_catalog_valid(
        catalog: &[DirectionalSprite],
        min_species: usize,
        expected_height: usize,
    ) {
        assert!(
            catalog.len() >= min_species,
            "expected at least {min_species} species, found {}",
            catalog.len()
        );

        for (i, species) in catalog.iter().enumerate() {
            assert!(!species.name.is_empty(), "species #{i} name must not be empty");
            assert_eq!(
                species.right.len(),
                expected_height,
                "species '{}' right height must be {expected_height}",
                species.name
            );
            assert_eq!(
                species.left.len(),
                expected_height,
                "species '{}' left height must be {expected_height}",
                species.name
            );

            let width = species.right[0].len();
            assert!(width > 0, "species '{}' width must be > 0", species.name);

            for (row, (right_line, left_line)) in
                species.right.iter().zip(species.left.iter()).enumerate()
            {
                assert_eq!(
                    right_line.len(),
                    width,
                    "species '{}' right row {row} length mismatch",
                    species.name
                );
                assert_eq!(
                    left_line.len(),
                    width,
                    "species '{}' left row {row} length mismatch",
                    species.name
                );
                assert!(
                    right_line.bytes().all(|b| (0x20..=0x7E).contains(&b)),
                    "species '{}' right row {row} must be printable ASCII",
                    species.name
                );
                assert!(
                    left_line.bytes().all(|b| (0x20..=0x7E).contains(&b)),
                    "species '{}' left row {row} must be printable ASCII",
                    species.name
                );
                assert_eq!(
                    *left_line,
                    mirror_ascii_line(right_line),
                    "species '{}' row {row} left sprite must be the exact horizontal mirror of right",
                    species.name
                );
            }

            for (j, other) in catalog.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        species.name, other.name,
                        "species #{i} and #{j} must have distinct names"
                    );
                    assert_ne!(
                        species.right, other.right,
                        "species '{}' and '{}' must have distinct art",
                        species.name, other.name
                    );
                }
            }
        }
    }

    fn assert_animated_catalog_valid(
        catalog: &[AnimatedSprite],
        min_species: usize,
        expected_height: usize,
    ) {
        assert!(
            catalog.len() >= min_species,
            "expected at least {min_species} animated species, found {}",
            catalog.len()
        );

        for (i, species) in catalog.iter().enumerate() {
            assert!(!species.name.is_empty(), "animated species #{i} name must not be empty");
            assert!(
                species.frames.len() >= 2,
                "animated species '{}' must have at least 2 frames",
                species.name
            );

            let width = species.frames[0][0].len();
            assert!(width > 0, "animated species '{}' width must be > 0", species.name);

            for (f_idx, frame) in species.frames.iter().enumerate() {
                assert_eq!(
                    frame.len(),
                    expected_height,
                    "animated species '{}' frame {f_idx} height must be {expected_height}",
                    species.name
                );
                for (row, line) in frame.iter().enumerate() {
                    assert_eq!(
                        line.len(),
                        width,
                        "animated species '{}' frame {f_idx} row {row} width mismatch",
                        species.name
                    );
                    assert!(
                        line.bytes().all(|b| (0x20..=0x7E).contains(&b)),
                        "animated species '{}' frame {f_idx} row {row} must be printable ASCII",
                        species.name
                    );
                }
            }

            assert_ne!(
                species.frames[0], species.frames[1],
                "animated species '{}' frames 0 and 1 must differ",
                species.name
            );
        }
    }

    #[test]
    fn medium_fish_catalog_has_valid_ascii_dimensions_and_mirroring() {
        assert_directional_catalog_valid(MEDIUM_FISH_SPECIES, 3, 3);
    }

    #[test]
    fn large_fish_catalog_has_shark_swordfish_whale_with_valid_ascii_and_mirroring() {
        assert_directional_catalog_valid(LARGE_FISH_SPECIES, 3, 5);
        let names: Vec<&str> = LARGE_FISH_SPECIES.iter().map(|s| s.name).collect();
        assert!(names.contains(&"shark"));
        assert!(names.contains(&"swordfish"));
        assert!(names.contains(&"whale"));
    }

    #[test]
    fn jellyfish_and_crab_animated_catalogs_have_valid_frames_and_ascii() {
        assert_animated_catalog_valid(JELLYFISH_SPECIES, 1, 4);
        assert_animated_catalog_valid(CRAB_SPECIES, 1, 2);
    }
}
