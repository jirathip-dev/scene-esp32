use artgen::{Backdrop, Slot, Sprite, Theme};

fn main() {
    embuild::espidf::sysenv::output();
    let farm = swag::packs().iter().find(|p| p.name == "farm");
    let farm = farm.unwrap_or_else(|| panic!("art: swag has no farm pack"));
    let out = std::env::var_os("OUT_DIR").unwrap_or_else(|| panic!("art: no OUT_DIR"));
    artgen::build(&FARM, farm, out.as_ref()).unwrap_or_else(|e| panic!("art: {e}"));
}

const fn horse(name: &'static str, symbol: &'static str) -> Sprite {
    Sprite {
        name,
        symbol,
        size: (66, 50),
        frames: &[],
    }
}

const FARM: Theme = Theme {
    backdrop: Backdrop {
        window: [0.0, 66.5, 390.0, 253.5],
        size: (320, 208),
        hide: &["sky-night"],
    },
    slots: &[
        ("--c", Slot::Coat),
        ("--d", Slot::CoatDark),
        ("--m", Slot::Mane),
        ("--b", Slot::Blaze),
        ("--fk", Slot::Fleet),
    ],
    sprites: &[
        horse("WORKING", "s-scene-edit"),
        horse("IDLE", "s-scene-idle"),
        horse("DONE", "s-scene-done"),
        horse("NEEDS", "s-scene-needs"),
        horse("FAIL", "s-scene-fail"),
        horse("LOADING", "s-loading"),
        horse("EMPTY", "s-empty"),
        Sprite {
            name: "BALE",
            symbol: "s-needs",
            size: (32, 24),
            frames: &["kf1", "kf2"],
        },
    ],
};
