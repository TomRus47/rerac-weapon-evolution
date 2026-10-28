//! Optional ballistic Pyrocitor stream. No placed actors, class imports or save data.
//!
//! A bounded set of droplets owns its launch-time tuning. Collision uses swept
//! native queries; the real-HP stream clock merges overlapping droplets into one
//! contact tick per target. Rendering is cosmetic and cannot gate damage or XP.
use super::{fx::PartSpawn, items::{HitSink, ItemEnv}, physics::{from_f32x3, line_world}};
use crate::{moby_runtime::MobyTable, moby_update::{real_hp::StreamSnapshot, services::HitTemplate}, ps2v::Pf};

const DT: f32 = 1.0 / 60.0;
const MAX_LANES: usize = 3;
const MAX_DROPLETS: usize = 120; // per lane; never shorten the centre stream
const MAX_IMPACT_PUFFS: usize = 12; // cosmetic work shared by all lanes per tick
const MAX_AGE: u16 = 120;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub enabled: bool,
    /// Additional DPS percentage, applied only when firing the lava replacement.
    pub damage_bonus_percent: f32,
    pub speed: f32,
    /// Elevation relative to the weapon aim, clamped at vertical.
    pub launch_angle_degrees: f32,
    pub gravity: f32,
    pub range: f32,
    pub size_percent: f32,
    pub core_color: Option<u32>,
    pub outer_color: Option<u32>,
    pub impact_color: Option<u32>,
    pub light_color: Option<u32>,
}
impl Default for Settings {
    fn default() -> Self {
        Self { enabled: false, damage_bonus_percent: 25.0, speed: 24.0, launch_angle_degrees: 10.0, gravity: 18.0, range: 18.0,
            size_percent: 100.0, core_color: None, outer_color: None,
            impact_color: None, light_color: None }
    }
}
impl Settings {
    pub fn valid(self) -> bool {
        [(self.damage_bonus_percent, 0.0, 1000.0), (self.speed, 1.0, 120.0), (self.launch_angle_degrees, -45.0, 45.0), (self.gravity, 0.0, 120.0),
         (self.range, 0.5, 100.0), (self.size_percent, 10.0, 500.0)]
            .into_iter().all(|(v, lo, hi)| v.is_finite() && (lo..=hi).contains(&v))
    }
    fn radius(self) -> f32 { 0.22 * self.size_percent / 100.0 }
}

#[derive(Clone, Debug, PartialEq)]
struct Droplet {
    lane: u8,
    pos: [f32; 3],
    previous: [f32; 3],
    velocity: [f32; 3],
    travelled: f32,
    age: u16,
    settings: Settings,
    damage: StreamSnapshot,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stream {
    droplets: Vec<Droplet>,
    /// Cosmetic polylines for the current simulation tick, never damage sources.
    strips: Vec<Vec<RenderPoint>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RenderPoint { pos: [f32; 3], settings: Settings }

impl Stream {
    pub fn clear(&mut self) { self.droplets.clear(); self.strips.clear(); }
    pub fn is_empty(&self) -> bool { self.droplets.is_empty() }
    pub fn launch(&mut self, pos: [f32; 3], direction: [f32; 3], settings: Settings, damage: StreamSnapshot) {
        self.launch_lane(0, pos, direction, settings, damage);
    }

    /// One launch tick, with a centre ray and optional independent side rays.
    /// Lane identity is retained in flight: interleaved launches must never be
    /// connected into a ribbon spanning the gaps between the three streams.
    pub fn launch_lanes(&mut self, pos: [f32; 3], directions: &[[f32; 3]], settings: Settings, damage: StreamSnapshot) {
        for (lane, &direction) in directions.iter().take(MAX_LANES).enumerate() {
            self.launch_lane(lane as u8, pos, direction, settings, damage);
        }
    }

    fn launch_lane(&mut self, lane: u8, pos: [f32; 3], direction: [f32; 3], settings: Settings, damage: StreamSnapshot) {
        let length = norm(direction);
        if !settings.valid() || length < 0.0001 || !length.is_finite()
            || !pos.into_iter().all(f32::is_finite)
            || self.droplets.iter().filter(|drop| drop.lane == lane).count() >= MAX_DROPLETS { return; }
        self.droplets.push(Droplet { lane, pos, previous: pos, velocity: launch_direction(direction, settings.launch_angle_degrees).map(|v| v * settings.speed),
            travelled: 0.0, age: 0, settings, damage });
    }

    /// Advance even after releasing fire, until impact, range expiry or put-away.
    pub fn step(&mut self, table: &mut MobyTable, env: &ItemEnv, hits: &mut dyn HitSink, parts: &mut Vec<PartSpawn>) {
        self.strips.clear();
        let mut impacts = Vec::new();
        self.droplets.retain_mut(|drop| {
            let start = drop.pos;
            let mut delta = drop.velocity.map(|v| v * DT);
            delta[2] -= 0.5 * drop.settings.gravity * DT * DT;
            drop.velocity[2] -= drop.settings.gravity * DT;
            let distance = norm(delta);
            let remaining = (drop.settings.range - drop.travelled).max(0.0);
            let fraction = if distance > 0.0 { (remaining / distance).min(1.0) } else { 1.0 };
            let intended = std::array::from_fn(|i| start[i] + delta[i] * fraction);
            let collision = match hits.probe(table, from_f32x3(start), from_f32x3(intended), 2, Some(env.hero_moby)) {
                Some(result) => result,
                None => env.coll.and_then(|world| line_world(world, from_f32x3(start), from_f32x3(intended), 2)).map(|hit| hit.point),
            };
            let end = collision.unwrap_or(intended);
            let actual = sub(end, start);
            // Clip before delivering: the stream must never damage through a wall.
            let direction = if norm(drop.velocity) > 0.0 { drop.velocity.map(|v| v / norm(drop.velocity)) } else { [0.0; 3] };
            let tmpl = HitTemplate { attacker: Some(env.hero_moby), flags: 0x10000, b18: 5, b19: 1, h1a: 176,
                damage: Pf::f(drop.damage.damage_hp_per_second / 100.0),
                dir: [Pf::f(direction[0]), Pf::f(direction[1]), Pf::ONE, Pf::b(0x45af_df66)], w20: 1 };
            hits.stream_snapshot(Some(drop.damage));
            let direct = hits.line(table, from_f32x3(start), from_f32x3(end), 2, Some(env.hero_moby), &tmpl).is_some();
            let steps = ((norm(actual) / (drop.settings.radius() * 0.75)).ceil() as usize).clamp(1, 32);
            let mut contact = false;
            for sample in 1..=steps {
                let t = sample as f32 / steps as f32;
                let point = std::array::from_fn(|i| start[i] + actual[i] * t);
                contact |= hits.sphere(table, Pf::f(drop.settings.radius()), from_f32x3(point), 0, Some(env.hero_moby), &tmpl).is_some();
            }
            hits.stream_snapshot(None);
            drop.previous = start;
            drop.travelled += norm(actual);
            drop.age += 1;
            drop.pos = end;
            let alive = collision.is_none() && !direct && !contact && fraction >= 1.0
                && drop.travelled < drop.settings.range && drop.age < MAX_AGE;
            if !alive {
                impacts.push(vec![RenderPoint { pos: start, settings: drop.settings }, RenderPoint { pos: end, settings: drop.settings }]);
                if impacts.len() <= MAX_IMPACT_PUFFS {
                    puff(parts, end, drop.settings.radius() * 1.8,
                        rgba(drop.settings.impact_color.unwrap_or(0x085AFF), 0x60), 8);
                }
            }
            alive
        });
        self.strips = impacts;
        // Adjacent flight samples become one connected strip. Break at a pause,
        // collision, recycled gap or large muzzle displacement instead of drawing
        // a long connector through geometry or between separate bursts.
        for lane in 0..MAX_LANES {
          let mut chain: Vec<RenderPoint> = Vec::new();
          let mut drops = self.droplets.iter().filter(|drop| drop.lane as usize == lane).peekable();
          while let Some(drop) = drops.next() {
            chain.push(RenderPoint { pos: drop.pos, settings: drop.settings });
            let joins_next = drops.peek().is_some_and(|next| {
                if drop.age != next.age.saturating_add(1) { return false; }
                let limit = ((norm(drop.velocity).max(norm(next.velocity)) * DT * 3.0)
                    + drop.settings.radius() + next.settings.radius()).clamp(0.5, 8.0);
                if norm(sub(drop.pos, next.pos)) > limit { return false; }
                match hits.probe(table, from_f32x3(drop.pos), from_f32x3(next.pos), 2, Some(env.hero_moby)) {
                    Some(hit) => hit.is_none(),
                    None => env.coll.and_then(|world| line_world(world, from_f32x3(drop.pos), from_f32x3(next.pos), 2)).is_none(),
                }
            });
            if !joins_next {
                chain.push(RenderPoint { pos: drop.previous, settings: drop.settings });
                self.strips.push(std::mem::take(&mut chain));
            }
          }
        }
    }

    /// Two camera-facing layers with shared vertices at every sample junction.
    /// Core/outer palettes and widths belong to each launched sample, so changing
    /// grade or colors does not recolor the stream already in flight.
    pub fn quads(&self, eye: [f32; 3]) -> Vec<super::tesla::BeamQuad> {
        let mut out = Vec::new();
        for (fx, width, alpha, core) in [(super::tesla::FX_GLOW, 1.5, 0x48, false),
                                       (super::tesla::FX_CORE, 0.65, 0x70, true)] {
            for strip in &self.strips {
                let mut edges: Vec<([f32; 3], [f32; 3], u32)> = Vec::with_capacity(strip.len());
                let mut previous_side = None;
                for (index, point) in strip.iter().enumerate() {
                    let before = strip[index.saturating_sub(1)].pos;
                    let after = strip[(index + 1).min(strip.len() - 1)].pos;
                    let tangent = sub(after, before);
                    let mut side = cross(tangent, sub(eye, point.pos));
                    if norm(side) < 0.00001 { side = cross(tangent, [0.0, 0.0, 1.0]); }
                    if norm(side) < 0.00001 { side = cross(tangent, [0.0, 1.0, 0.0]); }
                    side = if norm(side)<0.00001 { previous_side.unwrap_or([1.0,0.0,0.0]) } else { unit(side) };
                    if previous_side.is_some_and(|last: [f32; 3]| dot(last, side) < 0.0) { side = side.map(|v| -v); }
                    previous_side = Some(side);
                    let radius = point.settings.radius() * width;
                    let left = std::array::from_fn(|i| point.pos[i] + side[i] * radius);
                    let right = std::array::from_fn(|i| point.pos[i] - side[i] * radius);
                    let color = if core { point.settings.core_color.unwrap_or(0x5AD6FF) }
                        else { point.settings.outer_color.unwrap_or(0x0444F0) };
                    edges.push((left, right, rgba(color, alpha)));
                }
                for index in 0..edges.len().saturating_sub(1) {
                    if norm(sub(strip[index].pos, strip[index+1].pos)) < 0.00001 { continue; }
                    let (left, right, color) = edges[index];
                    let (next_left, next_right, next_color) = edges[index+1];
                    out.push(super::tesla::BeamQuad { fx, corners: [left, right, next_left, next_right],
                        // Sample one longitudinal slice: no repeating end fade at each joint.
                        st: [[0.5, 0.0], [0.5, 1.0], [0.5, 0.0], [0.5, 1.0]],
                        rgba: [color, color, next_color, next_color] });
                }
            }
        }
        out
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { std::array::from_fn(|i| a[i] - b[i]) }
fn norm(v: [f32; 3]) -> f32 { v.into_iter().map(|x| x*x).sum::<f32>().sqrt() }
fn rgba(rgb: u32, alpha: u8) -> u32 {
    // Native particles store R, G, B bytes in little-endian order, at PS2 intensity.
    let rgb = crate::weapon_evo::rgb_float(rgb).map(|v| (v * 127.0).round() as u32);
    (u32::from(alpha) << 24) | (rgb[2] << 16) | (rgb[1] << 8) | rgb[0]
}
fn puff(parts: &mut Vec<PartSpawn>, pos: [f32; 3], radius: f32, rgba: u32, life: i32) {
    parts.push(PartSpawn::Spark27 { size: radius * 210000.0,
        pos: [pos[0], pos[1], pos[2], 1.0], vel: [0.0; 4], rgba, life, rot: 0 });
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 { (0..3).map(|i| a[i]*b[i]).sum() }
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
}
fn unit(v: [f32; 3]) -> [f32; 3] { let len=norm(v); if len<0.00001 { [0.0;3] } else { v.map(|x|x/len) } }

/// Add pitch to the existing aim while keeping its azimuth and launch speed.
/// Clamp at vertical instead of reversing horizontal direction past the pole.
fn launch_direction(direction: [f32; 3], degrees: f32) -> [f32; 3] {
    let horizontal=direction[0].hypot(direction[1]);
    let pitch=(direction[2].atan2(horizontal)+degrees.to_radians())
        .clamp(-std::f32::consts::FRAC_PI_2,std::f32::consts::FRAC_PI_2);
    let (sin,cos)=pitch.sin_cos();
    let yaw=if horizontal>0.00001 {[direction[0]/horizontal,direction[1]/horizontal]} else {[1.0,0.0]};
    [yaw[0]*cos.max(0.0),yaw[1]*cos.max(0.0),sin]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hero::items::{NoHits, ItemData};
    use crate::pad::PadState;
    fn damage() -> StreamSnapshot { StreamSnapshot { damage_hp_per_second: 120.0, interval: 1, burn_color: Some(0xFF3300) } }
    fn environment<'a>(data: &'a ItemData, pad: &'a PadState) -> ItemEnv<'a> {
        ItemEnv { mobys: None, data, pad, frame: 1, hero_moby: 10, coll: None, camera: None, camera_up: None, targets: &[] }
    }
    #[test]
    fn gravity_range_expiry_and_release_tail_use_launch_snapshot() {
        let mut stream = Stream::default();
        let config = Settings { range: 2.0, launch_angle_degrees: 0.0, ..Default::default() };
        stream.launch([0.0; 3], [1.0, 0.0, 0.0], config, damage());
        let mut table = MobyTable::default(); let mut parts = Vec::new();
        let data = ItemData::default(); let pad = PadState::default();
        let env = environment(&data, &pad);
        stream.step(&mut table, &env, &mut NoHits, &mut parts);
        assert!(stream.droplets[0].pos[0] > 0.0);
        assert!(stream.droplets[0].pos[2] < 0.0);
        assert_eq!(stream.droplets[0].damage, damage());
        for _ in 0..5 { stream.step(&mut table, &env, &mut NoHits, &mut parts); }
        assert!(stream.is_empty(), "expiry is travelled distance, not hero height");
        assert!(!parts.is_empty()); assert!(table.mobys.is_empty());
    }
    #[derive(Default)]
    struct Wall { samples: Vec<f32>, scopes: Vec<Option<StreamSnapshot>> }
    impl HitSink for Wall {
        fn stream_snapshot(&mut self, settings: Option<StreamSnapshot>) { self.scopes.push(settings); }
        fn probe(&mut self, _: &mut MobyTable, _: [Pf;4], _: [Pf;4], _:u32, _:Option<usize>) -> Option<Option<[f32;3]>> {
            Some(Some([0.25, 0.0, 0.0]))
        }
        fn line(&mut self, _: &mut MobyTable, _: [Pf;4], end:[Pf;4], _:u32, _:Option<usize>, _: &HitTemplate) -> Option<Option<usize>> {
            assert!(end[0].to_f32() <= 0.25); Some(None)
        }
        fn sphere(&mut self, _: &mut MobyTable, _:Pf, centre:[Pf;4], _:u32, _:Option<usize>, _: &HitTemplate) -> Option<usize> {
            self.samples.push(centre[0].to_f32()); None
        }
    }
    #[test]
    fn terrain_clips_sweep_before_delivery_and_clears_snapshot_scope() {
        let mut stream=Stream::default();
        stream.launch([0.0;3], [1.0,0.0,0.0], Settings::default(), damage());
        let data=ItemData::default();let pad=PadState::default();let mut wall=Wall::default();let mut parts=Vec::new();
        stream.step(&mut MobyTable::default(), &environment(&data,&pad), &mut wall, &mut parts);
        assert!(stream.is_empty()); assert!(!wall.samples.is_empty());
        assert!(wall.samples.iter().all(|x| *x<=0.25));
        assert_eq!(wall.scopes, vec![Some(damage()),None]);
        assert!(parts.iter().any(|part| matches!(part,PartSpawn::Spark27{life:8,..})));
    }
    #[test]
    fn bounded_pool_and_clear_do_not_allocate_native_actors() {
        let mut stream = Stream::default();
        for _ in 0..1000 { stream.launch([0.0; 3], [1.0, 0.0, 0.0], Settings::default(), damage()); }
        assert_eq!(stream.droplets.len(), MAX_DROPLETS);
        stream.clear(); assert!(stream.is_empty());
        stream.launch([0.0; 3], [f32::NAN, 0.0, 0.0], Settings::default(), damage());
        assert!(stream.is_empty());
    }
    #[test]
    fn custom_particle_colors_are_rgb_and_do_not_use_global_flame_palette() {
        assert_eq!(rgba(crate::weapon_evo::parse_rgb("FF0000").unwrap().unwrap(), 0x70), 0x7000007F);
        assert_eq!(rgba(0x00FF00, 0x70), 0x70007F00);
        assert_eq!(rgba(crate::weapon_evo::parse_rgb("0000FF").unwrap().unwrap(), 0x70), 0x707F0000);
    }
    fn straight() -> Settings { Settings { gravity: 0.0, launch_angle_degrees: 0.0, ..Default::default() } }
    fn advance(stream: &mut Stream) {
        let data=ItemData::default();let pad=PadState::default();
        stream.step(&mut MobyTable::default(),&environment(&data,&pad),&mut NoHits,&mut Vec::new());
    }
    #[test]
    fn elevation_keeps_yaw_speed_and_clamps_without_flipping_at_vertical() {
        for angle in [-45.0,0.0,10.0,45.0] {
            let direction=launch_direction([0.0,5.0,0.0],angle);
            assert!(direction[0].abs()<0.00001);
            assert!((norm(direction)-1.0).abs()<0.00001);
            assert!((direction[2]-angle.to_radians().sin()).abs()<0.00001);
        }
        let up=launch_direction([0.0,1.0,1000.0],45.0);
        assert!(up[1]>=0.0);assert!(up[2]>0.999);
        let down=launch_direction([0.0,1.0,-1000.0],-45.0);
        assert!(down[1]>=0.0);assert!(down[2] < -0.999);
        let mut stream=Stream::default();
        stream.launch([0.0;3],[0.0,5.0,0.0],Settings::default(),damage());
        assert!((norm(stream.droplets[0].velocity)-24.0).abs()<0.0001);
        assert!(stream.droplets[0].velocity[2]>0.0);
    }
    #[test]
    fn continuous_strip_shares_joint_vertices_and_keeps_base_palette_and_width() {
        let mut stream=Stream::default();
        let settings=Settings{core_color:Some(0x0000FF),outer_color:Some(0x00FF00),..straight()};
        for _ in 0..3 { stream.launch([0.0;3],[1.0,0.0,0.0],settings,damage());advance(&mut stream); }
        let quads=stream.quads([0.0,-10.0,5.0]);
        assert_eq!(stream.strips.len(),1);assert_eq!(quads.len(),6);
        for layer in quads.chunks(3) {
            for pair in layer.windows(2) {
                assert_eq!(&pair[0].corners[2..],&pair[1].corners[..2]);
                assert_eq!(&pair[0].rgba[2..],&pair[1].rgba[..2]);
            }
            let scale=if layer[0].fx==super::super::tesla::FX_CORE {0.65} else {1.5};
            let expected=if scale==0.65 {rgba(0x0000FF,0x70)} else {rgba(0x00FF00,0x48)};
            for q in layer {
                assert!((norm(sub(q.corners[0],q.corners[1]))-0.44*scale).abs()<0.00001);
                assert_eq!(q.rgba,[expected;4]);
                assert_eq!(q.st[0],q.st[2]);assert_eq!(q.st[1],q.st[3]);
            }
        }
        stream.clear();assert!(stream.quads([0.0;3]).is_empty());
    }
    #[test]
    fn pause_breaks_the_strip_and_release_does_not_tether_it_to_the_nozzle() {
        let mut stream=Stream::default();
        stream.launch([0.0;3],[1.0,0.0,0.0],straight(),damage());advance(&mut stream);
        advance(&mut stream);
        assert!(stream.strips.iter().flatten().all(|point|point.pos[0]>0.0));
        stream.launch([0.0;3],[1.0,0.0,0.0],straight(),damage());advance(&mut stream);
        assert_eq!(stream.strips.len(),2);
        assert!(stream.strips.iter().all(|strip|strip.len()==2));
    }
    #[test]
    fn flight_palette_is_frozen_and_aligned_camera_does_not_produce_nan() {
        let mut stream=Stream::default();
        for color in [0x0000FF,0xFF0000] {
            stream.launch([0.0;3],[0.0,0.0,1.0],Settings{core_color:Some(color),size_percent:200.0,..straight()},damage());
            advance(&mut stream);
        }
        let quads=stream.quads([0.0,0.0,10.0]);
        assert!(quads.iter().flat_map(|q|q.corners).flatten().all(f32::is_finite));
        assert!(quads.iter().any(|q|q.rgba.contains(&rgba(0x0000FF,0x70))));
        assert!(quads.iter().any(|q|q.rgba.contains(&rgba(0xFF0000,0x70))));
        assert!(quads.iter().all(|q|norm(sub(q.corners[0],q.corners[1]))>0.0));
    }
    #[test]
    fn visible_stream_no_longer_spawns_a_chain_of_discrete_sprite_heads() {
        let mut stream=Stream::default();
        stream.launch([0.0;3],[1.0,0.0,0.0],straight(),damage());
        let data=ItemData::default();let pad=PadState::default();let mut parts=Vec::new();
        stream.step(&mut MobyTable::default(),&environment(&data,&pad),&mut NoHits,&mut parts);
        assert!(parts.is_empty());assert!(!stream.quads([0.0,-5.0,1.0]).is_empty());
    }

    #[test]
    fn three_interleaved_lanes_form_three_continuous_strips_without_cross_connections() {
        let mut stream = Stream::default();
        let directions = [[1.0,0.0,0.0], [0.8,-0.6,0.0], [0.8,0.6,0.0]];
        for _ in 0..12 {
            stream.launch_lanes([0.0;3], &directions, straight(), damage());
            advance(&mut stream);
        }
        assert_eq!(stream.droplets.len(),36);
        assert_eq!(stream.strips.len(),3);
        for (lane, strip) in stream.strips.iter().enumerate() {
            assert_eq!(strip.len(),13);
            for point in strip {
                let side = directions[lane][1] / directions[lane][0];
                assert!((point.pos[1] - point.pos[0] * side).abs() < 0.00001);
                assert_eq!(point.settings,straight());
            }
        }
        let quads = stream.quads([0.0,-10.0,5.0]);
        assert_eq!(quads.len(),72,"two layers times twelve segments times three lanes");
        // Turning Triple Flame off leaves the two existing tails ballistic;
        // it must neither tether them to nor join them with the centre ray.
        stream.launch([0.0;3],[1.0,0.0,0.0],straight(),damage()); advance(&mut stream);
        assert_eq!(stream.strips.len(),3);
        assert_eq!(stream.strips[0].len(),14);
        assert!(stream.strips[1..].iter().flatten().all(|p| p.pos[0]>0.0));
        stream.clear(); assert!(stream.is_empty() && stream.quads([0.0;3]).is_empty());
    }

    #[test]
    fn lane_budgets_are_fair_and_impact_particles_are_globally_bounded() {
        let mut stream = Stream::default();
        let directions = [[1.0,0.0,0.0];3];
        for _ in 0..1000 { stream.launch_lanes([0.0;3],&directions,straight(),damage()); }
        assert_eq!(stream.droplets.len(),MAX_LANES*MAX_DROPLETS);
        for lane in 0..MAX_LANES {
            assert_eq!(stream.droplets.iter().filter(|d| d.lane as usize==lane).count(),MAX_DROPLETS);
        }
        let data=ItemData::default();let pad=PadState::default();let mut wall=Wall::default();let mut parts=Vec::new();
        stream.step(&mut MobyTable::default(),&environment(&data,&pad),&mut wall,&mut parts);
        assert!(stream.is_empty());
        assert_eq!(parts.len(),MAX_IMPACT_PUFFS);
        assert_eq!(wall.scopes.len(),MAX_LANES*MAX_DROPLETS*2,"cosmetic cap never skips damage queries");
    }

    #[test]
    fn overlapping_lanes_retain_one_pyrocitor_contact_clock_per_target() {
        use crate::moby_update::{real_hp, services::HitTemplate};
        // Capture real stream queries and deliver them through the same real-HP
        // queue used by the engine. Three lanes cannot triple one target's DPS.
        struct Contacts { events: real_hp::Events, accepted: usize }
        impl HitSink for Contacts {
            fn stream_snapshot(&mut self,s:Option<StreamSnapshot>) { self.events.stream_snapshot=s; }
            fn probe(&mut self,_:&mut MobyTable,_:[Pf;4],_:[Pf;4],_:u32,_:Option<usize>)->Option<Option<[f32;3]>> {Some(None)}
            fn line(&mut self,_:&mut MobyTable,_:[Pf;4],_:[Pf;4],_:u32,_:Option<usize>,_:&HitTemplate)->Option<Option<usize>> {None}
            fn sphere(&mut self,table:&mut MobyTable,_:Pf,_:[Pf;4],_:u32,_:Option<usize>,hit:&HitTemplate)->Option<usize> {
                for target in 0..2 { assert!(self.events.queue(table,target,hit,false,[Pf::ZERO;4],0)); }
                self.accepted+=1;
                None
            }
        }
        let mut table=MobyTable::default();
        table.mobys=vec![crate::moby_runtime::Moby{o_class:577,state:1,..Default::default()};2];
        let mut events=real_hp::Events::default(); events.enabled=true; events.hero=Some(10);
        let mut contacts=Contacts{events,accepted:0}; let mut stream=Stream::default();
        let data=ItemData::default();let pad=PadState::default();let mut hp=[0.0;2];let mut pulses=[0;2];
        let damage=StreamSnapshot{interval:6,..damage()};
        for frame in 0..60 {
            contacts.events.begin_frame(frame);
            stream.launch_lanes([0.0;3],&[[1.0,0.0,0.0];3],straight(),damage);
            stream.step(&mut table,&environment(&data,&pad),&mut contacts,&mut Vec::new());
            for target in 0..2 {
                if let Some(hit)=contacts.events.take(target,u32::MAX) {
                    hp[target]+=hit.damage.to_f32()*real_hp::HP_SCALE; pulses[target]+=1;
                    assert_eq!(hit.evolution.unwrap().weapon,16);
                    assert_eq!(hit.evolution.unwrap().burn_color,Some(damage.burn_color));
                }
                assert!(contacts.events.take(target,u32::MAX).is_none());
            }
            assert!(contacts.events.stream_snapshot.is_none());
        }
        assert!(contacts.accepted>180,"overlapping old and new lane segments exercised");
        assert_eq!(pulses,[10;2]);
        assert!(hp.iter().all(|loss| (loss-120.0).abs()<0.001),"both targets get base DPS, not triple DPS: {hp:?}");
    }

}
