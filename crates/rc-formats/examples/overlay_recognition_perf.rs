//! Disc-free benchmark for repeated recognition against one immutable target.
//! Run with: cargo run --release -p rc-formats --example overlay_recognition_perf
use rc_formats::{font::OverlaySection, level_overlay::{LevelOverlay, Relocation, TEXT_SECTION}};
use std::{hint::black_box, time::Instant};

fn overlay(start: u32, code: &[u32]) -> LevelOverlay {
    let mut sections = vec![OverlaySection { dest: 0, kind: 1, entry: 0, data: Vec::new() }; TEXT_SECTION + 1];
    sections[TEXT_SECTION].dest = start;
    sections[TEXT_SECTION].data = code.iter().flat_map(|word| word.to_le_bytes()).collect();
    LevelOverlay::from_sections(sections)
}

fn main() {
    let body = [0x27bd_ffe0, 0xafbf_0010, 0x2402_1234, 0x2443_5678,
                0x0062_1021, 0x8fbf_0010, 0x03e0_0008, 0x27bd_0020, 0x0c09_0000, 0];
    let reference = overlay(0x24_0000, &body);
    let mut code: Vec<u32> = (0..131_072).map(|n| 0x2508_0000 | (n & 0xffff)).collect();
    code[512..512 + body.len()].copy_from_slice(&body);
    let target = overlay(0x28_0000, &code);
    let started = Instant::now();
    let mut checksum = 0u64;
    for _ in 0..32 {
        let relocation = Relocation::new(black_box(&reference), black_box(&target));
        assert_eq!(relocation.func(0x24_0000), Some(0x28_0800));
        checksum += relocation.func(0x24_0000).unwrap() as u64;
    }
    println!("{{\"target_words\":{},\"recognizers\":32,\"elapsed_ms\":{:.3},\"checksum\":{}}}",
             code.len(), started.elapsed().as_secs_f64() * 1000.0, black_box(checksum));
}
