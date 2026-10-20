//! Synthetic relocation fixtures: no extracted game data is needed.
use super::Overlay;
use rc_formats::{font::OverlaySection, level_overlay::{LevelOverlay, TEXT_SECTION}};

fn fixture(code: u32, data: u32, hi: u32, lo: u32) -> LevelOverlay {
    let mut sections = vec![OverlaySection { dest: 0, kind: 1, entry: 0, data: Vec::new() }; TEXT_SECTION + 1];
    sections[0].dest = data;
    sections[0].data = 42u32.to_le_bytes().to_vec();
    sections[TEXT_SECTION].dest = code;
    let words = [0x3c02_0000 | hi, 0x2442_0000 | lo, 0x2403_1234, 0x0062_1021,
                 0x03e0_0008, 0, 0x0c00_0000 | (code >> 2), 0];
    sections[TEXT_SECTION].data = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    LevelOverlay::from_sections(sections)
}

fn encode(overlay: &LevelOverlay) -> Vec<u8> {
    overlay.sections.iter().flat_map(|s| {
        let mut bytes: Vec<_> = [s.dest, s.data.len() as u32, s.kind, s.entry]
            .into_iter().flat_map(u32::to_le_bytes).collect();
        bytes.extend_from_slice(&s.data);
        bytes
    }).collect()
}

#[test]
fn parsed_menu_overlay_preserves_raw_address_maps_and_identity() {
    let reference = fixture(0x24_0000, 0x1c_22c0, 0x1c, 0x22c0);
    let target = fixture(0x28_0000, 0x2c_32c0, 0x2c, 0x32c0);
    let raw = Overlay::relocated(&encode(&target), &encode(&reference)).unwrap();
    let parsed = Overlay::relocated_parsed(&target, &reference);
    assert_eq!(raw.at(0x1c_22c0), 0x2c_32c0);
    for label in super::DATA_LABELS.iter().chain(super::vendor::layout::DATA_LABELS) {
        assert_eq!(parsed.at(*label), raw.at(*label));
        assert_eq!(parsed.maps(*label), raw.maps(*label));
    }
    assert_eq!(parsed.u32(parsed.at(0x1c_22c0)), Some(42));
    assert_eq!(parsed.label(0x28_0000), raw.label(0x28_0000));
    let identity = Overlay::relocated_parsed(&reference, &reference);
    assert_eq!(identity.at(0x1c_22c0), 0x1c_22c0);
    assert_eq!(identity.u32(0x1c_22c0), Some(42));
}
