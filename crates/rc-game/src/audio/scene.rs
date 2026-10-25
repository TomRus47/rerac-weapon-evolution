//! Audio of the in-engine scenes (mode 2, docs/plan/cutscenes_transitions.md §3): the speech VAG stream,
//! the music pause at the start and its resume after the end, and the cutscene master volumes.
//!
//! * Start (`DialogStreamStart` 0x2ac330): `music_Pause(0)`, then the dialogue player 0x151720 streams
//!   `scenes[id].speech[language]` (group 2) — 3 vsyncs before the first scene tick
//!   ([`crate::scene_player::AUDIO_LEAD`]).
//! * While the scene runs `sound_update` uses the cutscene master volumes (`voices::master_volumes`:
//!   music 0, groups 0 and 3 halved).
//! * End (0x2ac608): the speech stops (0x15172a → 5); `FUN_0027a460(ticks(30))` arms the paused music
//!   players' resume delay, so the music resumes 30 ticks later.
//!
//! The 989snd dialogue / pause internals are not reversed (inferred): the speech plays on one stream voice
//! at `rate_to_pitch(44056)` with the stream player's volume law at full volume; the pause freezes every
//! music stream voice (group 1, pitch and volume 0, position kept) and the resume restores them.
//!
//! **Hook.** The scene player runs in the engine's frame, the audio in its own driver, so requests go through
//! a process-wide inbox: [`post`] queues a [`SceneAudioCmd`]; the audio drains it at the start of its next
//! frame (the same 60 Hz frame when the scene system runs first).
//!
//! **Frames of a scene.** The engine suspends the gameplay tick while a scene runs, so the sound step of the
//! tick does not run either; [`AudioSystem::scene_frame_with`] is the audio frame of each scene frame: the
//! requests apply at once (the game issues them as calls from the blocking start / end code, not from
//! `sound_update`), then the EE frame (`sound_update`, `music_Update`: `CutsceneModeUpdate` runs them) when the
//! world runs this frame, else only the IOP / SPU (the blocking fades and vsync waits), then the 800 samples.
//!
//! **One music instance.** The pause holds every music stream voice, also one keyed while it lasts (a stream
//! command the render runs after the pause), until the resume; a second resume finds nothing to restore.

use super::music::{STREAM_ADSR1, STREAM_ADSR2};
use super::voices::{VoiceSlot, VoiceUse};
use super::{AudioSystem, StreamPart, VoiceData};
use rc_formats::vag;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Dialogue voice group (the dialogue player streams in group 2).
pub const SPEECH_GROUP: u8 = 2;
/// Music stream group.
pub const MUSIC_GROUP: u8 = 1;

/// A request from the scene player (`crate::scene_player::AudioRequest`, with the speech bytes resolved).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneAudioCmd {
    /// Scene start (`music_Pause(0)`): freeze the music voices, switch to the cutscene volumes.
    PauseMusic,
    /// Start the speech VAG (the whole file, header included).
    Speech { vag: Arc<[u8]> },
    /// Speech stop (0x15172a → 5) and the normal volumes.
    StopSpeech,
    /// `FUN_0027a460(after)`: the music resumes after `after` ticks.
    ResumeMusic { after: i32 },
    /// A reward without a cinematic: resume the held music when its one-shot
    /// ends. A later scene or menu takes ownership of the music hold instead.
    PickupJingle { vag: Arc<[u8]> },
}

static INBOX: Mutex<VecDeque<SceneAudioCmd>> = Mutex::new(VecDeque::new());

/// Tests that post to or drain the process-wide inbox hold this (the test harness runs them in parallel).
#[cfg(test)]
pub(crate) static TEST_INBOX: Mutex<()> = Mutex::new(());

/// Queues a request for the audio driver.
pub fn post(cmd: SceneAudioCmd) { INBOX.lock().unwrap_or_else(|e| e.into_inner()).push_back(cmd); }

/// Takes every queued request.
pub fn take() -> Vec<SceneAudioCmd> { INBOX.lock().unwrap_or_else(|e| e.into_inner()).drain(..).collect() }

/// A world pickup has no vendor menu to hold the level music. The native
/// vendor/demo stream supplies its non-looping acquisition jingle separately.
pub fn begin_weapon_pickup() {
    post(SceneAudioCmd::PauseMusic);
    post(SceneAudioCmd::StopSpeech);
}

/// Restore the held track; never restart track zero or allocate another music
/// player. This also stops a jingle that outlasts its acquisition scene.
pub fn end_weapon_pickup() {
    post(SceneAudioCmd::StopSpeech);
    post(SceneAudioCmd::ResumeMusic { after: 0 });
}

/// The scene audio state kept by [`AudioSystem`].
#[derive(Clone, Debug, Default)]
pub struct SceneAudio {
    /// Mode 2: `sound_update` uses the cutscene master volumes.
    pub cutscene: bool,
    /// The speech voice and its key-on generation.
    pub speech: Option<(usize, u32)>,
    /// Between `PauseMusic` and the resume: every music stream voice is held.
    pub music_paused: bool,
    pickup_auto_resume: bool,
    /// Frozen music voices: (voice, generation, pitch, volume registers).
    paused: Vec<(usize, u32, u16, [u16; 2])>,
    /// Ticks until the music resumes (`FUN_0027a460`).
    pub resume_in: Option<i32>,
    /// Requests applied so far (reports / tests).
    pub applied: u32,
    /// `snd_PauseAllSoundsInGroup`'s mask of paused 989snd groups (the page menu's 0x1d), and the voices it froze:
    /// (voice, generation, pitch).
    pub groups_paused: u32,
    group_frozen: Vec<(usize, u32, u16)>,
}

/// The 989snd groups the page menu holds (`EnterMenuMode` 0x28bf50): 0, 2, 3, 4; group 1 (the music) is held by
/// `music_Pause` instead.
pub const MENU_PAUSED_GROUPS: u32 = 0x1d;

impl AudioSystem {
    /// A pickup demo freezes the world, including the gameplay audio hook.
    /// Drain its requests and advance the audio once per demo frame anyway.
    /// No enemy updates, emitters, music-box requests or gameplay RNG run here.
    pub fn weapon_pickup_frame(&mut self, out: &mut Vec<[i16; 2]>) {
        self.scene_frame();
        self.music_resume_countdown();
        self.render(super::SAMPLES_PER_FRAME, out);
    }

    /// Applies one scene request (called by `game_frame` for the inbox; callable directly).
    pub fn scene_command(&mut self, cmd: SceneAudioCmd) {
        self.scene.applied += 1;
        // A story scene, another pickup or a stop/resume request supersedes a
        // banner-only reward. Release its hold before the new owner acts.
        if std::mem::take(&mut self.scene.pickup_auto_resume) {
            self.stop_speech();
            self.music_unpause();
        }
        match cmd {
            SceneAudioCmd::PauseMusic => {
                self.scene.cutscene = true;
                self.scene.resume_in = None;
                self.scene.music_paused = true;
                self.pause_music();
            }
            SceneAudioCmd::Speech { vag } => {
                self.stop_speech();
                self.start_speech(&vag);
            }
            SceneAudioCmd::StopSpeech => {
                self.stop_speech();
                self.scene.cutscene = false;
            }
            SceneAudioCmd::ResumeMusic { after } => {
                self.scene.cutscene = false;
                self.scene.resume_in = Some(after.max(0));
            }
            SceneAudioCmd::PickupJingle { vag } => {
                self.scene_command(SceneAudioCmd::PauseMusic);
                self.scene_command(SceneAudioCmd::Speech { vag });
                self.scene.cutscene = false;
                self.scene.pickup_auto_resume = true;
            }
        }
    }

    /// `music_Pause(0)` outside a scene (the vendor's `OpenVendorMenu`): the music voices are held, the volumes are the
    /// normal ones.
    pub fn music_pause(&mut self) {
        if std::mem::take(&mut self.scene.pickup_auto_resume) { self.stop_speech(); }
        self.scene.music_paused = true;
        self.pause_music();
    }

    /// `snd_PauseAllSoundsInGroup(mask)` (boot 0x12e3e8, 989snd command 0x16; the page menu's `EnterMenuMode` 0x28bf50
    /// with mask 0x1d = groups 0, 2, 3, 4): every sound of those groups stops where it is: its handler stops running
    /// grains and LFOs, its voices hold (pitch and volume 0, as OpenGOAL's `VoiceManager::Pause`).
    pub fn pause_groups(&mut self, mask: u32) {
        self.snd.set_groups_paused(mask, true);
        for (i, slot) in self.snd.vm.voices.iter().enumerate() {
            let v = &mut self.spu.voices[i];
            if mask & (1 << (slot.group & 31)) == 0 || !v.active() || slot.generation != v.generation { continue; }
            if self.scene.group_frozen.iter().any(|f| f.0 == i && f.1 == v.generation) { continue; }
            self.scene.group_frozen.push((i, v.generation, v.pitch));
            v.pitch = 0;
            v.vol = [0, 0];
        }
        self.scene.groups_paused |= mask;
    }

    /// `snd_ContinueAllSoundsInGroup(mask)` (boot 0x12e418, command 0x17; `PageMenuUpdate`'s close): the held voices take
    /// their pitch and their volume (the voice's `MakeVolume` through its group) back, the handlers run again.
    pub fn continue_groups(&mut self, mask: u32) {
        self.snd.set_groups_paused(mask, false);
        let frozen = std::mem::take(&mut self.scene.group_frozen);
        for (i, generation, pitch) in frozen {
            let slot = self.snd.vm.voices[i];
            if mask & (1 << (slot.group & 31)) == 0 {
                self.scene.group_frozen.push((i, generation, pitch));
                continue;
            }
            let v = &mut self.spu.voices[i];
            if v.generation != generation { continue; }
            v.pitch = pitch;
            v.vol = self.snd.vm.voice_registers(slot.basevol, slot.group as usize);
        }
        self.scene.groups_paused &= !mask;
    }

    /// `EnterMenuMode` (0x28bf50) and `OpenVendorMenu` (0x2ae1a0): `snd_PauseAllSoundsInGroup(0x1d)` then
    /// `music_Pause(0)` (then `snd_FlushSoundCommands`: the commands here apply at once).
    pub fn menu_open(&mut self) {
        self.pause_groups(MENU_PAUSED_GROUPS);
        self.music_pause();
    }

    /// The page menu's close (`PageMenuUpdate` 0x28c990 once kind 0x14 ends) and `VendorExit` (0x2ae660):
    /// `snd_ContinueAllSoundsInGroup(0x1d)`, then
    /// `music_Unpause` unless the post-action is the ship travel (`unpause_music` false: the music stays paused into the
    /// flight), then the flush and `sound_update` (the caller's audio frame).
    pub fn menu_close(&mut self, unpause_music: bool) {
        self.continue_groups(MENU_PAUSED_GROUPS);
        if unpause_music { self.music_unpause(); }
    }

    /// `music_Unpause` (the vendor's `VendorExit`): the held music voices resume at once.
    pub fn music_unpause(&mut self) {
        if self.scene.music_paused { self.resume_music(); }
    }

    /// Applies the queued requests ([`post`]) now.
    pub fn scene_inbox(&mut self) {
        for c in take() { self.scene_command(c); }
    }

    /// One audio frame of a mode-2 scene frame while the gameplay tick is suspended (module docs): the
    /// requests, then the EE frame on the game's stream ([`AudioSystem::game_frame_with`]) when `world_runs`,
    /// then the 800 samples appended to `out`.
    pub fn scene_frame_with(
        &mut self,
        world_runs: bool,
        input: &super::FrameInput,
        frame: u32,
        rng: &mut crate::rng::Rng,
        owner_pos: &dyn Fn(u32) -> Option<[f32; 3]>,
        out: &mut Vec<[i16; 2]>,
    ) {
        self.scene_inbox();
        if world_runs { self.game_frame_with(input, frame, rng, owner_pos); }
        self.render(super::SAMPLES_PER_FRAME, out);
    }

    /// Per EE frame, before `sound_update`: the inbox.
    pub(super) fn scene_frame(&mut self) {
        self.scene_inbox();
        if self.scene.pickup_auto_resume && !self.speech_playing() {
            self.scene.pickup_auto_resume = false;
            self.stop_speech();
            self.music_unpause();
        }
    }

    /// The resume countdown: `music_UpdateStream` 0x27a4a0's `FastDecTimer(+0xe)` on a paused player (state & 0x8000)
    /// inside `music_Update`, so every frame that runs `sound_update` counts it: gameplay, the page menus and the
    /// credits slideshow (mode 7), which the end of the game enters within the 30 ticks after its last scene.
    pub(super) fn music_resume_countdown(&mut self) {
        if let Some(t) = self.scene.resume_in {
            if t <= 0 {
                self.scene.resume_in = None;
                self.resume_music();
            } else {
                self.scene.resume_in = Some(t - 1);
            }
        }
    }

    /// Freezes the music stream voices. The render calls it again after its commands while the pause lasts:
    /// a stream keyed meanwhile is held too, and a held voice whose registers were rewritten (the group volume
    /// update when the cutscene volumes end, a stream volume change) keeps the new values for the resume and
    /// stays silent.
    pub(super) fn pause_music(&mut self) {
        let vm = &self.snd.vm;
        for (i, slot) in vm.voices.iter().enumerate() {
            let v = &mut self.spu.voices[i];
            if slot.group != MUSIC_GROUP || !matches!(slot.owner, VoiceUse::Stream { .. }) || !v.active() || slot.generation != v.generation { continue; }
            match self.scene.paused.iter_mut().find(|p| p.0 == i && p.1 == v.generation) {
                Some(p) => {
                    if v.pitch != 0 { p.2 = v.pitch; }
                    if v.vol != [0, 0] { p.3 = v.vol; }
                }
                None => {
                    self.scene.paused.retain(|p| p.0 != i);
                    self.scene.paused.push((i, v.generation, v.pitch, v.vol));
                }
            }
            v.pitch = 0;
            v.vol = [0, 0];
        }
    }

    fn resume_music(&mut self) {
        self.scene.music_paused = false;
        for (i, generation, pitch, vol) in std::mem::take(&mut self.scene.paused) {
            let v = &mut self.spu.voices[i];
            if v.generation == generation {
                v.pitch = pitch;
                v.vol = vol;
            }
        }
    }

    fn start_speech(&mut self, bytes: &[u8]) {
        let Ok((h, body)) = vag::parse_vag(bytes) else { return };
        let Ok(ext) = vag::sample_extent(body, 0) else { return };
        let part = StreamPart { data: Arc::from(&body[..ext.bytes()]), looped: false };
        let vm = &mut self.snd.vm;
        let Some(i) = vm.allocate(&mut self.spu, SPEECH_GROUP as usize, 127, 0) else { return };
        let base = vm.make_volume(127, 0, 127, 0, 127, 0);
        let regs = vm.voice_registers(base, SPEECH_GROUP as usize);
        let v = &mut self.spu.voices[i];
        v.key_on(VoiceData::Stream { cur: part, queue: Default::default() }, 0, vag::rate_to_pitch(h.sample_rate), STREAM_ADSR1, STREAM_ADSR2);
        v.vol = regs;
        let generation = v.generation;
        vm.voices[i] = VoiceSlot { owner: VoiceUse::Stream { handle: u32::MAX }, priority: 127, group: SPEECH_GROUP, basevol: base, start_tick: 0, generation };
        self.scene.speech = Some((i, generation));
    }

    fn stop_speech(&mut self) {
        if let Some((i, generation)) = self.scene.speech.take() {
            if self.spu.voices[i].generation == generation { self.spu.voices[i].key_off(); }
            self.snd.vm.voices[i].owner = VoiceUse::Free;
        }
    }

    /// Music stream voices (group 1) sounding and held by the pause: (playing, paused).
    pub fn music_voices(&self) -> (usize, usize) {
        let (mut playing, mut held) = (0, 0);
        for (i, slot) in self.snd.vm.voices.iter().enumerate() {
            let v = &self.spu.voices[i];
            if slot.group != MUSIC_GROUP || !matches!(slot.owner, VoiceUse::Stream { .. }) || !v.active() || slot.generation != v.generation { continue; }
            if self.scene.paused.iter().any(|p| p.0 == i && p.1 == v.generation) { held += 1 } else { playing += 1 }
        }
        (playing, held)
    }

    /// Whether the speech voice is still sounding.
    pub fn speech_playing(&self) -> bool {
        self.scene.speech.is_some_and(|(i, g)| self.spu.voices[i].generation == g && self.spu.voices[i].active())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_pickup_audio() -> (AudioSystem, Arc<[u8]>) {
        use super::super::LevelAudio;
        use rc_formats::sound_bank::{Bank, LevelSounds, SfxHeader};
        let data = LevelAudio {
            bank: Arc::new(Bank { file_type: 3, chunks: [(0, 0); 2],
                header: SfxHeader::default(), sounds: vec![], grains: vec![],
                vags: vec![], samples: vec![] }),
            sounds: LevelSounds::default(), instances: vec![], instance_pvars: vec![],
            music: std::array::from_fn(|_| None), env_points: vec![], collision: None,
        };
        let mut sys = AudioSystem::new(data);
        let mut body = vec![0x11; 16 * 128];
        for frame in body.chunks_exact_mut(16) { frame[0] = 12; frame[1] = 0; }
        body[16 * 127 + 1] = vag::FLAG_END | vag::FLAG_REPEAT;
        let voice = &mut sys.spu.voices[0];
        voice.key_on(VoiceData::Stream { cur: StreamPart {
            data: Arc::from(body.clone()), looped: true }, queue: Default::default() },
            0, 0x1000, STREAM_ADSR1, STREAM_ADSR2);
        voice.vol = [0x2000; 2];
        sys.snd.vm.voices[0] = VoiceSlot { owner: VoiceUse::Stream { handle: 77 },
            priority: 127, group: MUSIC_GROUP, basevol: (0x2000, 0x2000), start_tick: 0,
            generation: voice.generation };
        sys.music.main.track = 6; sys.music.main.handle = 77;
        let mut jingle = vec![0; vag::VAG_HEADER_SIZE];
        jingle[..4].copy_from_slice(b"VAGp");
        jingle[12..16].copy_from_slice(&(body.len() as u32).to_be_bytes());
        jingle[16..20].copy_from_slice(&48_000u32.to_be_bytes());
        jingle.extend_from_slice(&body);
        (sys, Arc::from(jingle))
    }

    #[test]
    fn pickup_audio_runs_without_world_ticks_plays_once_and_restores_the_held_track() {
        let _lock = TEST_INBOX.lock().unwrap_or_else(|e| e.into_inner());
        take();
        let (mut sys, jingle) = synthetic_pickup_audio();
        let held = sys.spu.voices[0].stream_remaining();
        let generation = sys.spu.voices[0].generation;
        begin_weapon_pickup();
        let mut out = Vec::new();
        sys.weapon_pickup_frame(&mut out);
        assert_eq!(out.len(), super::super::SAMPLES_PER_FRAME);
        assert_eq!(sys.music_voices(), (0, 1));
        post(SceneAudioCmd::Speech { vag: jingle });
        sys.weapon_pickup_frame(&mut Vec::new());
        assert!(sys.speech_playing());
        let jingle_voice = sys.scene.speech;
        for _ in 0..20 {
            sys.weapon_pickup_frame(&mut Vec::new());
            assert_eq!(sys.music_voices(), (0, 1));
            assert_eq!(sys.spu.voices[0].stream_remaining(), held);
            assert_eq!(sys.scene.speech, jingle_voice, "jingle restarted");
        }
        assert!(!sys.speech_playing(), "one-shot must finish before the scene ends");
        end_weapon_pickup();
        sys.weapon_pickup_frame(&mut Vec::new());
        assert_eq!(sys.music_voices(), (1, 0));
        assert!(sys.scene.speech.is_none());
        assert_eq!(sys.spu.voices[0].generation, generation);
        assert_eq!((sys.music.main.track, sys.music.main.handle), (6, 77));
        assert!(sys.spu.voices[0].stream_remaining() < held);
        assert!(take().is_empty());
    }

    #[test]
    fn ending_a_short_pickup_stops_a_jingle_that_has_not_finished() {
        let _lock = TEST_INBOX.lock().unwrap_or_else(|e| e.into_inner());
        take();
        let (mut sys, jingle) = synthetic_pickup_audio();
        begin_weapon_pickup(); post(SceneAudioCmd::Speech { vag: jingle });
        sys.weapon_pickup_frame(&mut Vec::new());
        assert!(sys.speech_playing());
        end_weapon_pickup(); sys.weapon_pickup_frame(&mut Vec::new());
        assert!(sys.scene.speech.is_none());
        assert!(!sys.scene.music_paused);
        assert_eq!(sys.music_voices(), (1, 0));
    }

    #[test]
    fn banner_only_reward_resumes_music_when_the_jingle_ends_and_accepts_another_pickup() {
        let _lock = TEST_INBOX.lock().unwrap_or_else(|e| e.into_inner());
        take();
        let (mut sys, jingle) = synthetic_pickup_audio();
        let generation = sys.spu.voices[0].generation;
        for _ in 0..2 {
            post(SceneAudioCmd::PickupJingle { vag: jingle.clone() });
            sys.weapon_pickup_frame(&mut Vec::new());
            assert!(sys.speech_playing());
            assert_eq!(sys.music_voices(), (0, 1));
            for _ in 0..20 { sys.weapon_pickup_frame(&mut Vec::new()); }
            assert!(!sys.scene.music_paused);
            assert!(sys.scene.speech.is_none());
            assert_eq!(sys.music_voices(), (1, 0));
            assert_eq!(sys.spu.voices[0].generation, generation);
        }
    }

    #[test]
    fn a_story_or_menu_takes_over_a_banner_jingle_without_premature_music_resume() {
        let _lock = TEST_INBOX.lock().unwrap_or_else(|e| e.into_inner());
        take();
        for menu in [false, true] {
            let (mut sys, jingle) = synthetic_pickup_audio();
            post(SceneAudioCmd::PickupJingle { vag: jingle });
            sys.weapon_pickup_frame(&mut Vec::new());
            if menu { sys.music_pause(); }
            else { sys.scene_command(SceneAudioCmd::PauseMusic); }
            for _ in 0..20 { sys.weapon_pickup_frame(&mut Vec::new()); }
            assert!(sys.scene.music_paused);
            assert_eq!(sys.music_voices(), (0, 1));
            sys.music_unpause();
            assert_eq!(sys.music_voices(), (1, 0));
        }
    }

    #[test]
    fn missing_jingle_data_never_leaves_level_music_paused() {
        let _lock = TEST_INBOX.lock().unwrap_or_else(|e| e.into_inner());
        take();
        let (mut sys, _) = synthetic_pickup_audio();
        post(SceneAudioCmd::PickupJingle { vag: Arc::from(&b"invalid"[..]) });
        sys.weapon_pickup_frame(&mut Vec::new());
        assert!(!sys.scene.music_paused);
        assert_eq!(sys.music_voices(), (1, 0));
    }

    #[test]
    fn inbox_keeps_order() {
        let _lock = TEST_INBOX.lock().unwrap_or_else(|e| e.into_inner());
        let vag: Arc<[u8]> = Arc::from(&b"VAGp"[..]);
        post(SceneAudioCmd::PauseMusic);
        post(SceneAudioCmd::Speech { vag: vag.clone() });
        post(SceneAudioCmd::ResumeMusic { after: 30 });
        let got = take();
        let mine: Vec<_> = got.into_iter().filter(|c| matches!(c, SceneAudioCmd::PauseMusic | SceneAudioCmd::Speech { .. } | SceneAudioCmd::ResumeMusic { after: 30 })).collect();
        assert_eq!(mine, [SceneAudioCmd::PauseMusic, SceneAudioCmd::Speech { vag }, SceneAudioCmd::ResumeMusic { after: 30 }]);
        assert!(take().is_empty());
    }

    /// The scene command path on Novalis (extracted data; skipped without it), frame by frame as the engine
    /// drives a scene: level start (music track 0), `PauseMusic` → no music voice sounds (one held), the speech
    /// streams, the EE frames of the scene queue the loop body behind the held voice (still one), `StopSpeech`,
    /// `ResumeMusic { after: 30 }` → held for 30 more EE frames, then exactly one music voice; a second resume
    /// adds none, and the music keeps one voice across the Start → Loop hand-over.
    #[test]
    fn scene_commands_keep_one_music_instance() {
        use super::super::{FrameInput, LevelAudio};
        let _lock = TEST_INBOX.lock().unwrap_or_else(|e| e.into_inner());
        let root = rc_formats::test_data::root().join("levels/01");
        let Ok(bank) = std::fs::read(root.join("sound_bank.bin")) else { eprintln!("skipped: no extracted/"); return };
        let rd = |n: &str| std::fs::read(root.join(n)).unwrap();
        let idx = rd("core_index.bin");
        let data = rc_formats::test_data::core_data(1).unwrap();
        let core = rc_formats::level::parse_level_core(&idx, data.len()).unwrap();
        let music: [Option<Vec<u8>>; 15] = std::array::from_fn(|k| std::fs::read(root.join(format!("music/{k:03}.bin"))).ok());
        let audio = LevelAudio::from_parts(&bank, &idx, &core, &data, &rc_formats::test_data::gameplay(1).unwrap(), &rd("level_header.bin"), &music, None).unwrap();
        let speech: Arc<[u8]> = Arc::from(rd("speech/05_en.bin"));
        let input = FrameInput { hero_pos: [162.5, 136.4, 60.5], ..Default::default() };
        let mut sys = AudioSystem::new(audio);
        let mut rng = crate::rng::Rng::new();
        rng.srand(crate::rng::LEVEL_SEED);
        let mut out = Vec::new();
        let mut counter = 1u32;
        take();
        // Gameplay tick 1: music_start_track(0) and its key-on.
        sys.tick_with(&input, counter, &mut rng, &|_| None, &mut out);
        counter += 1;
        assert_eq!(sys.music_voices(), (1, 0));
        let frame = |sys: &mut AudioSystem, world: bool, cmd: Option<SceneAudioCmd>, rng: &mut crate::rng::Rng, counter: &mut u32| {
            if let Some(c) = cmd { post(c); }
            sys.scene_frame_with(world, &input, *counter, rng, &|_| None, &mut Vec::new());
            if world { *counter += 1; }
        };
        // Scene start (a frozen fade frame): the music is held at once.
        frame(&mut sys, false, Some(SceneAudioCmd::PauseMusic), &mut rng, &mut counter);
        assert_eq!(sys.music_voices(), (0, 1));
        assert!(sys.scene.cutscene && sys.scene.music_paused);
        let held = sys.spu.voices.iter().map(|v| v.stream_remaining()).collect::<Vec<_>>();
        frame(&mut sys, false, Some(SceneAudioCmd::Speech { vag: speech }), &mut rng, &mut counter);
        assert!(sys.speech_playing());
        // Scene ticks: music_Update takes the start callback and queues the loop behind the held voice.
        for _ in 0..600 {
            frame(&mut sys, true, None, &mut rng, &mut counter);
            assert_eq!(sys.music_voices(), (0, 1));
        }
        assert!(sys.music.main.state == 4 && sys.music.main.handle != 0, "{:?}", sys.music.main);
        assert!(sys.speech_playing());
        // The held voice did not advance.
        let music_voice = (0..sys.spu.voices.len()).find(|&i| sys.snd.vm.voices[i].group == MUSIC_GROUP && sys.spu.voices[i].active()).unwrap();
        assert_eq!(sys.spu.voices[music_voice].stream_remaining(), held[music_voice]);
        // End: speech stop, the resume armed on the closing frame.
        frame(&mut sys, false, Some(SceneAudioCmd::StopSpeech), &mut rng, &mut counter);
        assert!(sys.scene.speech.is_none() && !sys.scene.cutscene);
        frame(&mut sys, false, Some(SceneAudioCmd::ResumeMusic { after: 30 }), &mut rng, &mut counter);
        // Back in gameplay: 30 ticks held (silent, although the normal group volumes are written back), then
        // one music voice.
        for _ in 0..30 {
            sys.tick_with(&input, counter, &mut rng, &|_| None, &mut out);
            counter += 1;
            assert_eq!(sys.music_voices(), (0, 1));
            assert_eq!((sys.spu.voices[music_voice].pitch, sys.spu.voices[music_voice].vol), (0, [0, 0]));
        }
        sys.tick_with(&input, counter, &mut rng, &|_| None, &mut out);
        counter += 1;
        assert_eq!(sys.music_voices(), (1, 0));
        assert!(!sys.scene.music_paused);
        assert!(sys.spu.voices[music_voice].pitch != 0 && sys.spu.voices[music_voice].vol != [0, 0]);
        // A second resume restores nothing and starts nothing; the Start → Loop hand-over keeps one voice.
        post(SceneAudioCmd::ResumeMusic { after: 30 });
        let start_secs = sys.data.music[0].as_ref().unwrap().seconds();
        let frames = (start_secs * 60.0) as usize + 600;
        for _ in 0..frames {
            sys.tick_with(&input, counter, &mut rng, &|_| None, &mut out);
            counter += 1;
            assert_eq!(sys.music_voices(), (1, 0));
        }
        assert!(sys.music.main.state == 4 && sys.music.main.handle != 0, "{:?}", sys.music.main);
    }

    /// The end of the game: the last scene (Veldin's boss, scene 5) ends with `FUN_0027a460(ticks(30))` and the boss enters
    /// the credits slideshow (mode 7) at once, whose frames run only `sound_update`. The resume delay counts down there
    /// (`music_UpdateStream`'s `FastDecTimer(+0xe)`), so the music comes back 30 frames in; it did not (silent credits).
    #[test]
    fn music_resumes_during_slideshow_frames() {
        use super::super::{FrameInput, LevelAudio};
        let _lock = TEST_INBOX.lock().unwrap_or_else(|e| e.into_inner());
        let root = rc_formats::test_data::root().join("levels/01");
        let Ok(bank) = std::fs::read(root.join("sound_bank.bin")) else { eprintln!("skipped: no extracted/"); return };
        let rd = |n: &str| std::fs::read(root.join(n)).unwrap();
        let idx = rd("core_index.bin");
        let data = rc_formats::test_data::core_data(1).unwrap();
        let core = rc_formats::level::parse_level_core(&idx, data.len()).unwrap();
        let music: [Option<Vec<u8>>; 15] = std::array::from_fn(|k| std::fs::read(root.join(format!("music/{k:03}.bin"))).ok());
        let audio = LevelAudio::from_parts(&bank, &idx, &core, &data, &rc_formats::test_data::gameplay(1).unwrap(), &rd("level_header.bin"), &music, None).unwrap();
        let input = FrameInput { hero_pos: [162.5, 136.4, 60.5], ..Default::default() };
        let mut sys = AudioSystem::new(audio);
        let mut rng = crate::rng::Rng::new();
        rng.srand(crate::rng::LEVEL_SEED);
        let mut out = Vec::new();
        let mut counter = 1u32;
        take();
        sys.tick_with(&input, counter, &mut rng, &|_| None, &mut out);
        counter += 1;
        assert_eq!(sys.music_voices(), (1, 0));
        // The scene: the music held, then its end arms the resume.
        for cmd in [SceneAudioCmd::PauseMusic, SceneAudioCmd::ResumeMusic { after: 30 }] {
            post(cmd);
            sys.scene_frame_with(false, &input, counter, &mut rng, &|_| None, &mut Vec::new());
        }
        assert_eq!(sys.music_voices(), (0, 1));
        // Slideshow frames: `sound_update` and the samples, no gameplay frame.
        let slide = |sys: &mut AudioSystem, rng: &mut crate::rng::Rng| {
            let mut buf = Vec::new();
            sys.sound_update_with(&input, counter, rng, &|_| None);
            sys.render(crate::audio::SAMPLES_PER_FRAME, &mut buf);
        };
        for _ in 0..30 {
            slide(&mut sys, &mut rng);
            assert_eq!(sys.music_voices(), (0, 1));
        }
        slide(&mut sys, &mut rng);
        assert!(!sys.scene.music_paused);
        assert_eq!(sys.music_voices(), (1, 0));
    }

    /// The page menu's audio (`EnterMenuMode` 0x28bf50 / `PageMenuUpdate` 0x28c990's close) on Novalis: the open holds the
    /// music (`music_Pause`) and every sounding voice of groups 0, 2, 3, 4 (pitch and volume 0, the handler stopped);
    /// class 0x472's five menu sounds each get a slot and sound while the menu is open; the close gives the held voices
    /// their pitch back and resumes the music at once, except for the ship travel, which leaves the music held.
    #[test]
    fn menu_open_holds_music_and_world_sounds_and_plays_menu_sounds() {
        use super::super::{FrameInput, LevelAudio};
        use crate::menus::MenuSound;
        let _lock = TEST_INBOX.lock().unwrap_or_else(|e| e.into_inner());
        let root = rc_formats::test_data::root().join("levels/01");
        let Ok(bank) = std::fs::read(root.join("sound_bank.bin")) else { eprintln!("skipped: no extracted/"); return };
        let rd = |n: &str| std::fs::read(root.join(n)).unwrap();
        let idx = rd("core_index.bin");
        let data = rc_formats::test_data::core_data(1).unwrap();
        let core = rc_formats::level::parse_level_core(&idx, data.len()).unwrap();
        let music: [Option<Vec<u8>>; 15] = std::array::from_fn(|k| std::fs::read(root.join(format!("music/{k:03}.bin"))).ok());
        let audio = LevelAudio::from_parts(&bank, &idx, &core, &data, &rc_formats::test_data::gameplay(1).unwrap(), &rd("level_header.bin"), &music, None).unwrap();
        let input = FrameInput { hero_pos: [162.5, 136.4, 60.5], ..Default::default() };
        let listener = input.listener;
        let mut rng = crate::rng::Rng::new();
        rng.srand(crate::rng::LEVEL_SEED);
        let mut out = Vec::new();
        let mut counter = 1u32;
        take();
        let fresh = |rng: &mut crate::rng::Rng, counter: &mut u32, out: &mut Vec<[i16; 2]>| {
            let mut sys = AudioSystem::new(audio.clone());
            sys.tick_with(&input, *counter, rng, &|_| None, out);
            *counter += 1;
            assert_eq!(sys.music_voices(), (1, 0));
            sys
        };
        let frame = |sys: &mut AudioSystem, rng: &mut crate::rng::Rng, counter: &mut u32| {
            sys.tick_with(&input, *counter, rng, &|_| None, &mut Vec::new());
            *counter += 1;
        };
        // A page-menu frame: `sound_update` alone (`SceneController` 0x28c990), then the samples.
        let menu_frame = |sys: &mut AudioSystem, rng: &mut crate::rng::Rng, counter: &mut u32| {
            sys.sound_update_with(&input, *counter, rng, &|_| None);
            sys.render(super::super::SAMPLES_PER_FRAME, &mut Vec::new());
            *counter += 1;
        };
        // The sounding voices of a 989snd group mask (index, pitch).
        let sounding = |sys: &AudioSystem, mask: u32| -> Vec<(usize, u16)> {
            (0..sys.spu.voices.len())
                .filter(|&i| {
                    let (slot, v) = (&sys.snd.vm.voices[i], &sys.spu.voices[i]);
                    mask & (1 << (slot.group & 31)) != 0 && v.active() && slot.generation == v.generation && !matches!(slot.owner, super::super::voices::VoiceUse::Stream { .. })
                })
                .map(|i| (i, sys.spu.voices[i].pitch))
                .collect()
        };
        let mut sys = fresh(&mut rng, &mut counter, &mut out);
        // A world sound (the page-change sound, played as the world would) sounding before the open.
        assert!(sys.play_class_sound(&MenuSound::PageChange.event(counter as u64), None, &listener, &mut rng) >= 0);
        frame(&mut sys, &mut rng, &mut counter);
        frame(&mut sys, &mut rng, &mut counter);
        let before = sounding(&sys, MENU_PAUSED_GROUPS);
        assert!(!before.is_empty() && before.iter().all(|&(_, p)| p != 0), "{before:?}");
        // Open: the music held, the world voices silent and still.
        sys.menu_open();
        assert_eq!(sys.music_voices(), (0, 1));
        assert_eq!(sys.scene.groups_paused, MENU_PAUSED_GROUPS);
        assert_eq!(sys.snd.paused_groups, MENU_PAUSED_GROUPS);
        for &(i, _) in &before {
            assert_eq!((sys.spu.voices[i].pitch, sys.spu.voices[i].vol), (0, [0, 0]), "voice {i}");
        }
        // Each menu sound while open: a slot, then a sounding voice of its own (not held).
        for s in [MenuSound::Confirm, MenuSound::Cursor, MenuSound::Denied, MenuSound::Open, MenuSound::PageChange] {
            let k = sys.play_class_sound(&s.event(counter as u64), None, &listener, &mut rng);
            assert!(k >= 0, "{s:?} got no slot");
            assert_eq!(sys.slots.slots[k as usize].class_index, s as u16);
            menu_frame(&mut sys, &mut rng, &mut counter);
            menu_frame(&mut sys, &mut rng, &mut counter);
            let now = sounding(&sys, !0);
            assert!(now.iter().any(|&(i, p)| p != 0 && !before.iter().any(|b| b.0 == i)), "{s:?}: no voice sounding: {now:?}");
            for &(i, _) in &before {
                if sys.snd.vm.voices[i].generation == sys.spu.voices[i].generation {
                    assert_eq!(sys.spu.voices[i].pitch, 0, "{s:?}: held voice {i} ran");
                }
            }
            assert_eq!(sys.music_voices(), (0, 1), "{s:?}");
        }
        // Close (any post-action but the ship travel): the held voices take their pitch back, the music plays.
        sys.menu_close(true);
        assert_eq!((sys.scene.groups_paused, sys.snd.paused_groups), (0, 0));
        assert_eq!(sys.music_voices(), (1, 0));
        for &(i, p) in &before {
            if sys.snd.vm.voices[i].generation == sys.spu.voices[i].generation && sys.spu.voices[i].active() {
                assert_eq!(sys.spu.voices[i].pitch, p, "voice {i}");
                assert_ne!(sys.spu.voices[i].vol, [0, 0], "voice {i}");
            }
        }
        menu_frame(&mut sys, &mut rng, &mut counter);
        assert_eq!(sys.music_voices(), (1, 0));
        // The ship travel's close: the groups continue, the music stays held.
        let mut sys = fresh(&mut rng, &mut counter, &mut out);
        sys.menu_open();
        sys.menu_close(false);
        assert_eq!(sys.scene.groups_paused, 0);
        assert_eq!(sys.music_voices(), (0, 1));
        frame(&mut sys, &mut rng, &mut counter);
        assert_eq!(sys.music_voices(), (0, 1));
    }
}
