//! "Where is X on level N": the level overlay's sections and class table, and a relocator that finds, on any
//! level, the function or data a level-01 (reference) address names, by matching the code
//! (docs/plan/level_generalisation.md X1/X2).
//!
//! **Layout.** Every level overlay (`extracted/levels/NN/overlay.bin`, [`crate::font::parse_overlay_sections`])
//! has seven sections in the same order on all 19 levels: `.lit` (0x15ef00), `.bss`, `.data`, `lvl.vtbl`,
//! `lvl.camvtbl`, `lvl.sndvtbl`, `.text` (checked against the Lombyte section tables). `lvl.vtbl` is the class
//! → update table: 12-byte records `{i32 o_class, u32 update, u32 _}` ending at `o_class == -1`
//! ([`LevelOverlay::vtbl`]).
//!
//! **Code identity across overlays.** The engine code is the same object code in every overlay, linked at other
//! addresses, so the same function differs only in relocated fields. [`mask`] hides exactly those: `j`/`jal`
//! targets, `lui` immediates of main-RAM addresses (0x0010..0x0040, so float and small-integer `lui`s stay
//! compared), and the 16-bit immediates of the instructions that complete such a `lui` (`%lo`, tracked per
//! register, also through `move`) or address `$gp` ([`GP`], the same on every level). Two functions are "the same code" when their masked words
//! are equal over the reference function's extent (from its start to the next known function start: `jal`
//! targets, class-table entries, code addresses formed with `lui`/`%lo`, and a stack-frame set-up right after a
//! `jr ra`). This is the port's notion of "the level's copy of a level-01 function";
//! a false negative (a real copy not recognised) only loses a port, a false positive would need two functions
//! identical up to their relocations.
//!
//! **Relocation.** [`Relocation`] maps a reference (level-01) function to its copy in the target overlay
//! ([`Relocation::func`]), and a reference data address to the target's by finding a reference function that
//! forms the address with `lui`/`%lo` and reading the same instruction pair in that function's copy
//! ([`Relocation::data`]). The port keeps its level-01 addresses as labels and asks the relocator for the loaded
//! level's.

use crate::font::{parse_overlay_sections, read_overlay, OverlaySection};
use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

type CodeIndex = HashMap<[u32; 4], Vec<u32>>;
type AddressSites = HashMap<u32, Vec<(u32, usize)>>;

/// Index of `lvl.vtbl` in the overlay's section list (the same on all 19 levels).
pub const VTBL_SECTION: usize = 3;
/// Index of `lvl.camvtbl` (the camera classes, [`LevelOverlay::camvtbl`]).
pub const CAMVTBL_SECTION: usize = 4;
/// Index of `.text`.
pub const TEXT_SECTION: usize = 6;
/// `$gp` in every level (the boot ELF sets it once; the small data is addressed from it: `0x15f5c4` is
/// `-0x763c($gp)` in all 19 overlays).
pub const GP: u32 = 0x166c00;
/// Longest function extent compared (bytes); a longer reference extent is cut here.
pub const MAX_EXTENT: u32 = 0x8000;

/// One `lvl.vtbl` record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VtblEntry {
    pub o_class: i32,
    /// The update function (0: none).
    pub update: u32,
    pub w8: u32,
}

/// One `lvl.camvtbl` record (0x14 bytes): a camera class and its four functions (`UpdateAllCameras` 0x20d620 reads
/// them by the UpdateCam's mode +0x8c, the record's index: activate +4 (`Camera_ActivationCheckPriority` 0x20d410),
/// init +8 (the switch `0x20d110`), update +0xc, pre +0x10 (the current camera's, before the activation loop)).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CamVtblEntry {
    pub class: i32,
    pub activate: u32,
    pub init: u32,
    pub update: u32,
    pub pre: u32,
}

/// A parsed level overlay.
#[derive(Clone, Debug)]
pub struct LevelOverlay {
    pub sections: Arc<Vec<OverlaySection>>,
    text_start: u32,
    text: Vec<u32>,
    starts: BTreeSet<u32>,
    /// Recognition data depends only on the private immutable text, not on a
    /// particular reference/target pair. Share it across all recognizers and
    /// initialized overlay clones, and build it only when actually requested.
    code_index: Arc<Mutex<Option<Arc<CodeIndex>>>>,
    address_sites: Arc<Mutex<Option<Arc<AddressSites>>>>,
}

const fn op(w: u32) -> u32 { w >> 26 }
const fn rs(w: u32) -> usize { ((w >> 21) & 31) as usize }
const fn rt(w: u32) -> usize { ((w >> 16) & 31) as usize }
const fn rd(w: u32) -> usize { ((w >> 11) & 31) as usize }
fn sext(w: u32) -> u32 { (w & 0xffff) as u16 as i16 as i32 as u32 }

/// Branches (not relocated: relative offsets).
fn is_branch(o: u32) -> bool { matches!(o, 1 | 4..=7 | 0x14..=0x17) }
/// I-type instructions that write GPR `rt`.
fn writes_rt(o: u32) -> bool { matches!(o, 8..=15 | 0x18..=0x1b | 0x1e | 0x20..=0x27 | 0x37) }
/// `lui` of a main-RAM address (the only ones a relink changes).
fn lui_is_address(w: u32) -> bool { (0x0010..0x0040).contains(&(w & 0xffff)) }

/// Per-register "holds a `lui` of an address" state while walking code in address order. Calls and branches
/// do not reset it (a `lui` in a branch's delay slot is used at the branch target, after code that may call):
/// a register keeps its state until an instruction writes it.
#[derive(Clone, Copy, Default)]
struct HiState([Option<u32>; 32]);

impl HiState {
    /// Masks `w` and updates the state; returns (masked word, the full address `w` forms from a `lui`, if any).
    fn step(&mut self, w: u32) -> (u32, Option<u32>) { self.step_inner(w) }

    fn step_inner(&mut self, w: u32) -> (u32, Option<u32>) {
        let o = op(w);
        match o {
            // j / jal: the target.
            2 | 3 => (w & 0xfc00_0000, None),
            15 => {
                let r = rt(w);
                if r == 0 { return (w, None); }
                if lui_is_address(w) {
                    self.0[r] = Some(w & 0xffff);
                    (w & 0xffff_0000, None)
                } else {
                    self.0[r] = None;
                    (w, None)
                }
            }
            0 => {
                let f = w & 63;
                let (s, t, d) = (rs(w), rt(w), rd(w));
                // move (addu / daddu / or with $zero) keeps the state; any other write clears it.
                let moved = if matches!(f, 0x21 | 0x2d | 0x25) {
                    if t == 0 { self.0[s] } else if s == 0 { self.0[t] } else { None }
                } else {
                    None
                };
                if d != 0 { self.0[d] = moved; }
                (w, None)
            }
            _ if is_branch(o) => (w, None),
            0x10..=0x13 => {
                // Coprocessor moves to a GPR (mfc / dmfc / qmfc / cfc: rs 0..2) write rt.
                if rs(w) <= 2 && rt(w) != 0 { self.0[rt(w)] = None; }
                (w, None)
            }
            0x1c => {
                // MMI: writes rd.
                if rd(w) != 0 { self.0[rd(w)] = None; }
                (w, None)
            }
            _ => {
                let s = rs(w);
                // `ori` completes a `lui` with the zero-extended immediate, the others sign-extend it.
                let lo = if o == 13 { w & 0xffff } else { sext(w) };
                let full = if s == 28 { Some(GP.wrapping_add(sext(w))) } else { self.0[s].map(|hi| (hi << 16).wrapping_add(lo)) };
                let masked = if full.is_some() { w & 0xffff_0000 } else { w };
                if writes_rt(o) && rt(w) != 0 { self.0[rt(w)] = None; }
                (masked, full)
            }
        }
    }
}

/// The masked words of `words` (module doc), walked from a function start.
pub fn mask(words: &[u32]) -> Vec<u32> { walk(words).into_iter().map(|(m, _)| m).collect() }

/// The (word offset, address) of every `lui`/`%lo` pair completed in `words`, and of every `$gp`-relative access.
pub fn address_refs(words: &[u32]) -> Vec<(usize, u32)> {
    walk(words).into_iter().enumerate().filter_map(|(i, (_, a))| a.map(|a| (i, a))).collect()
}

/// The masked first four words (the relocator's index key; equal to the first four of [`mask`] of any longer run).
fn key(w: &[u32]) -> [u32; 4] {
    let m = walk(&w[..4]);
    [m[0].0, m[1].0, m[2].0, m[3].0]
}

/// Walks `words` from a function start in address order; at a forward branch target the state also takes the
/// registers the branch brought in (after its delay slot), so a `lui` in a delay slot reaches its use.
fn walk(words: &[u32]) -> Vec<(u32, Option<u32>)> {
    let mut st = HiState::default();
    let mut pending: HashMap<usize, HiState> = HashMap::new();
    let mut branch_to: Option<usize> = None;
    let mut out = Vec::with_capacity(words.len());
    for (i, &w) in words.iter().enumerate() {
        if let Some(p) = pending.remove(&i) {
            for r in 0..32 { if st.0[r].is_none() { st.0[r] = p.0[r]; } }
        }
        out.push(st.step(w));
        if let Some(t) = branch_to.take() {
            // The delay slot has run: the branch's state reaches its target.
            let e = pending.entry(t).or_default();
            for r in 0..32 { if e.0[r].is_none() { e.0[r] = st.0[r]; } }
        }
        let o = op(w);
        if is_branch(o) || (o == 0x11 && rs(w) == 8) {
            let t = i as i64 + 1 + (w & 0xffff) as u16 as i16 as i64;
            if t > i as i64 + 1 && (t as usize) < words.len() { branch_to = Some(t as usize); }
        }
    }
    out
}

impl LevelOverlay {
    pub fn parse(bytes: &[u8]) -> crate::buf::Result<LevelOverlay> { Ok(Self::from_sections(parse_overlay_sections(bytes)?)) }

    pub fn from_sections(sections: Vec<OverlaySection>) -> LevelOverlay {
        let (text_start, text) = sections
            .get(TEXT_SECTION)
            .map(|s| (s.dest, s.data.as_chunks::<4>().0.iter().map(|c| u32::from_le_bytes(*c)).collect()))
            .unwrap_or_default();
        let mut ov = LevelOverlay { sections: Arc::new(sections), text_start, text, starts: BTreeSet::new(), code_index: Default::default(), address_sites: Default::default() };
        let end = ov.text_end();
        let mut starts: BTreeSet<u32> = ov
            .text
            .iter()
            .filter(|&&w| op(w) == 3)
            .map(|&w| (w & 0x03ff_ffff) << 2)
            .filter(|&a| a >= ov.text_start && a < end)
            .collect();
        starts.extend(ov.vtbl().iter().map(|e| e.update).filter(|&a| a >= ov.text_start && a < end));
        // A frame set-up right after a return (`jr ra` + delay slot, then `addiu sp, sp, -n`), and code addresses
        // formed with `lui`/`%lo` (callbacks handed to other code).
        for p in 2..ov.text.len() {
            let w = ov.text[p];
            if ov.text[p - 2] == 0x03e0_0008 && w >> 16 == 0x27bd && w & 0x8000 != 0 { starts.insert(ov.text_start + 4 * p as u32); }
        }
        starts.extend(address_refs(&ov.text).into_iter().map(|(_, a)| a).filter(|&a| a >= ov.text_start && a < end && a % 4 == 0));
        ov.starts = starts;
        ov
    }

    /// `n` bytes at `addr` (None past the loaded sections).
    pub fn read(&self, addr: u32, n: usize) -> Option<&[u8]> { read_overlay(&self.sections, addr, n) }
    pub fn u32(&self, addr: u32) -> Option<u32> { self.read(addr, 4).map(|b| u32::from_le_bytes(b.try_into().unwrap())) }

    fn text_end(&self) -> u32 { self.text_start + 4 * self.text.len() as u32 }

    fn code_index(&self) -> Arc<CodeIndex> {
        self.code_index.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(|| {
            let mut index = HashMap::<_, Vec<_>>::new();
            for p in 0..self.text.len().saturating_sub(3) {
                index.entry(key(&self.text[p..p + 4])).or_default().push(self.text_start + 4 * p as u32);
            }
            Arc::new(index)
        }).clone()
    }

    fn address_sites(&self) -> Arc<AddressSites> {
        self.address_sites.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(|| {
            let mut sites = HashMap::<_, Vec<_>>::new();
            for &f in &self.starts {
                let Some(code) = self.extent(f).and_then(|n| self.code(f, n)) else { continue };
                for (offset, address) in address_refs(code) { sites.entry(address).or_default().push((f, offset)); }
            }
            Arc::new(sites)
        }).clone()
    }

    /// Release temporary recognition maps after loader preparation. Parsed
    /// overlays and resolved tables stay valid; in-flight readers retain their
    /// Arc leases. A later recognizer can rebuild the same maps lazily.
    pub fn clear_recognition_cache(&self) {
        self.code_index.lock().unwrap_or_else(|e| e.into_inner()).take();
        self.address_sites.lock().unwrap_or_else(|e| e.into_inner()).take();
    }

    /// `words` code words at `addr` of `.text`.
    pub fn code(&self, addr: u32, words: usize) -> Option<&[u32]> {
        let i = addr.checked_sub(self.text_start)?;
        if i % 4 != 0 { return None; }
        let i = (i / 4) as usize;
        self.text.get(i..i.checked_add(words)?)
    }

    /// The `lvl.vtbl` records up to the `-1` end marker.
    pub fn vtbl(&self) -> Vec<VtblEntry> {
        let Some(s) = self.sections.get(VTBL_SECTION) else { return Vec::new() };
        s.data
            .as_chunks::<12>()
            .0
            .iter()
            .map(|e| VtblEntry {
                o_class: i32::from_le_bytes(e[0..4].try_into().unwrap()),
                update: u32::from_le_bytes(e[4..8].try_into().unwrap()),
                w8: u32::from_le_bytes(e[8..12].try_into().unwrap()),
            })
            .take_while(|e| e.o_class != -1)
            .collect()
    }

    /// The `lvl.camvtbl` records up to the `-1` end marker (the slot init's class → mode lookup `0x20ee30` searches
    /// them; class 0 is always the first).
    pub fn camvtbl(&self) -> Vec<CamVtblEntry> {
        let Some(s) = self.sections.get(CAMVTBL_SECTION) else { return Vec::new() };
        let w = |e: &[u8], i: usize| u32::from_le_bytes(e[4 * i..4 * i + 4].try_into().unwrap());
        s.data
            .as_chunks::<0x14>()
            .0
            .iter()
            .map(|e| CamVtblEntry { class: w(e, 0) as i32, activate: w(e, 1), init: w(e, 2), update: w(e, 3), pre: w(e, 4) })
            .take_while(|e| e.class != -1)
            .collect()
    }

    /// The code extent of the function at `addr`, in words: up to the next known function start (`jal` targets,
    /// class-table updates), at most [`MAX_EXTENT`] bytes and the end of `.text`.
    pub fn extent(&self, addr: u32) -> Option<usize> {
        let end = self.text_end();
        if addr < self.text_start || addr >= end { return None; }
        let next = self.starts.range(addr + 4..).next().copied().unwrap_or(end);
        Some((next.min(addr.saturating_add(MAX_EXTENT)).min(end) - addr) as usize / 4)
    }
}

/// Reference → target address translation (module doc).
pub struct Relocation<'a> {
    reference: &'a LevelOverlay,
    target: &'a LevelOverlay,
    identity: bool,
}

impl<'a> Relocation<'a> {
    /// Identity when the target is the reference itself (same bytes).
    pub fn new(reference: &'a LevelOverlay, target: &'a LevelOverlay) -> Relocation<'a> {
        let identity = reference.text_start == target.text_start && reference.text == target.text && reference.sections == target.sections;
        Relocation { reference, target, identity }
    }

    /// Whether the target is the reference overlay itself.
    pub fn is_identity(&self) -> bool { self.identity }

    /// The reference function at `ref_fn`, masked over its extent.
    fn ref_masked(&self, ref_fn: u32) -> Option<Vec<u32>> {
        let n = self.reference.extent(ref_fn)?;
        Some(mask(self.reference.code(ref_fn, n)?))
    }

    /// Whether the target code at `target_fn` is the reference function `ref_fn` (module doc).
    pub fn same_code(&self, ref_fn: u32, target_fn: u32) -> bool {
        if self.identity { return ref_fn == target_fn; }
        let Some(m) = self.ref_masked(ref_fn) else { return false };
        self.target.code(target_fn, m.len()).is_some_and(|c| mask(c) == m)
    }

    /// Every copy of the reference function `ref_fn` in the target's `.text`.
    pub fn copies(&self, ref_fn: u32) -> Vec<u32> {
        if self.identity { return self.reference.code(ref_fn, 1).map(|_| vec![ref_fn]).unwrap_or_default(); }
        let Some(m) = self.ref_masked(ref_fn).filter(|m| m.len() >= 4) else { return Vec::new() };
        let index = self.target.code_index();
        let Some(cands) = index.get(&[m[0], m[1], m[2], m[3]]) else { return Vec::new() };
        cands.iter().copied().filter(|&c| self.target.code(c, m.len()).is_some_and(|w| mask(w) == m)).collect()
    }

    /// The target's copy of the reference function `ref_fn` (None: not found, or found more than once).
    pub fn func(&self, ref_fn: u32) -> Option<u32> {
        if self.identity { return Some(ref_fn); }
        match self.copies(ref_fn).as_slice() {
            [a] => Some(*a),
            _ => None,
        }
    }

    /// The target's address for the reference data address `ref_addr` (module doc). None when no reference
    /// function forms it with `lui`/`%lo`, or none of those functions has a copy in the target.
    pub fn data(&self, ref_addr: u32) -> Option<u32> {
        if self.identity { return Some(ref_addr); }
        let address_sites = self.reference.address_sites();
        let sites = address_sites.get(&ref_addr)?;
        for &(f, off) in sites {
            let Some(g) = self.func(f) else { continue };
            let n = self.reference.extent(f)?;
            let refs = address_refs(self.target.code(g, n)?);
            if let Some(&(_, a)) = refs.iter().find(|(o, _)| *o == off) { return Some(a); }
        }
        // No copy of a whole referencing function (its extent runs into level code): the code around one of
        // the references, from its function start up to 16 words past it, found once in the target.
        for &(f, off) in sites {
            let n = (off + 16).min(self.reference.extent(f)?);
            let m = mask(self.reference.code(f, n)?);
            if m.len() < 4 { continue; }
            let index = self.target.code_index();
            let Some(cands) = index.get(&[m[0], m[1], m[2], m[3]]) else { continue };
            let hits: Vec<u32> = cands.iter().copied().filter(|&c| self.target.code(c, n).is_some_and(|w| mask(w) == m)).collect();
            if let [g] = hits.as_slice() {
                if let Some(&(_, a)) = address_refs(self.target.code(*g, n)?).iter().find(|(o, _)| *o == off) { return Some(a); }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overlay(start: u32, text: &[u32]) -> LevelOverlay {
        let mut sections = vec![OverlaySection { dest: 0, kind: 1, entry: 0, data: Vec::new() }; TEXT_SECTION + 1];
        sections[TEXT_SECTION].dest = start;
        sections[TEXT_SECTION].data = text.iter().flat_map(|w| w.to_le_bytes()).collect();
        LevelOverlay::from_sections(sections)
    }

    const BODY: [u32; 8] = [0x27bd_fff0, 0xafbf_0000, 0x2402_1234, 0x2443_5678,
                             0x03e0_0008, 0x27bd_0010, 0x0c09_0000, 0];

    #[test]
    fn shared_recognition_preserves_all_copies_and_ambiguity() {
        let reference = overlay(0x24_0000, &BODY);
        let code: Vec<_> = [BODY.as_slice(), &[0; 8], BODY.as_slice()].concat();
        let target = overlay(0x28_0000, &code);
        let masked = mask(reference.code(0x24_0000, reference.extent(0x24_0000).unwrap()).unwrap());
        let expected: Vec<_> = code.windows(masked.len()).enumerate()
            .filter(|(_, words)| mask(words) == masked)
            .map(|(i, _)| 0x28_0000 + 4 * i as u32).collect();
        assert_eq!(expected.len(), 2);
        for _ in 0..3 {
            let relocation = Relocation::new(&reference, &target);
            assert_eq!(relocation.copies(0x24_0000), expected);
            assert_eq!(relocation.func(0x24_0000), None);
        }
        let other = overlay(0x2c_0000, &code);
        assert_eq!(Relocation::new(&reference, &other).copies(0x24_0000),
                   expected.iter().map(|a| a + 0x4_0000).collect::<Vec<_>>());
    }

    #[test]
    fn recognition_is_lazy_and_initialized_clones_share_the_index() {
        let reference = overlay(0x24_0000, &BODY);
        let target = overlay(0x28_0000, &BODY);
        let relocation = Relocation::new(&reference, &target);
        assert!(target.code_index.lock().unwrap().is_none());
        assert!(relocation.same_code(0x24_0000, 0x28_0000));
        assert!(target.code_index.lock().unwrap().is_none());
        assert_eq!(relocation.func(0x24_0000), Some(0x28_0000));
        let cloned = target.clone();
        assert!(Arc::ptr_eq(&target.code_index(), &cloned.code_index()));
        assert!(reference.address_sites.lock().unwrap().is_none());
        assert_eq!(relocation.data(0x1234), None);
        let cloned_reference = reference.clone();
        assert!(Arc::ptr_eq(&reference.address_sites(), &cloned_reference.address_sites()));
        assert!(Relocation::new(&reference, &reference).is_identity());
        let lease = target.code_index();
        let weak = Arc::downgrade(&lease);
        target.clear_recognition_cache();
        assert!(cloned.code_index.lock().unwrap().is_none());
        assert!(weak.upgrade().is_some());
        drop(lease);
        assert!(weak.upgrade().is_none());
        assert_eq!(relocation.func(0x24_0000), Some(0x28_0000));
        reference.clear_recognition_cache();
        assert!(reference.address_sites.lock().unwrap().is_none());
    }

    #[test]
    fn loader_and_consumers_can_share_recognition_across_threads() {
        let reference = overlay(0x24_0000, &BODY);
        let target = overlay(0x28_0000, &BODY);
        std::thread::scope(|scope| {
            for _ in 0..8 {
                scope.spawn(|| assert_eq!(Relocation::new(&reference, &target).func(0x24_0000), Some(0x28_0000)));
            }
        });
        assert!(target.code_index.lock().unwrap().is_some());
    }

    #[test]
    fn masking_follows_lui_through_moves_and_keeps_constants() {
        // lui v0,0x17 ; addiu s1,v0,0x42c0 ; move s2,v0 ; addiu s1,s2,0x42c0 ; lui at,0x3f80 ; lw t0,8(a0) ; lw t1,-0x10(gp)
        let w = [0x3c02_0017, 0x2451_42c0, 0x0040_902d, 0x2651_42c0, 0x3c01_3f80, 0x8c88_0008, 0x8f89_fff0];
        let m = mask(&w);
        assert_eq!(m, vec![0x3c02_0000, 0x2451_0000, 0x0040_902d, 0x2651_0000, 0x3c01_3f80, 0x8c88_0008, 0x8f89_0000]);
        assert_eq!(address_refs(&w), vec![(1, 0x1742c0), (3, 0x1742c0), (6, GP - 0x10)]);
        // lui a0,0x1b ; addiu a0,a0,-0x1000 → 0x1af000
        assert_eq!(address_refs(&[0x3c04_001b, 0x2484_f000]), vec![(1, 0x1a_f000)]);
    }
}
