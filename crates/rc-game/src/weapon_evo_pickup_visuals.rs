//! Presentation rules shared by randomized pickups and vendor previews.
//! No source completion, inventory, damage or progression changes live here.

pub const INFOBOT_CLASS: i32 = 750;
pub const AMMO_CRATE_CLASS:i32=511;
pub const GOLD_BOLT_CLASS: i32 = 1134;
pub const HALO_SEGMENTS: usize = 24;
pub const HALO_CENTRE_RGBA: u32 = 0x2840_ffff;
pub const HALO_EDGE_RGBA: u32 = 0x0040_ffff;

/// A display completes one turn every 240 logic ticks. Reduce the integer
/// clock BEFORE converting to f32, so long sessions retain the same phase.
pub fn spin_angle(ticks: u64) -> f32 {
    (ticks % 240) as f32 * std::f32::consts::TAU / 240.
}

/// Native VU sine/cosine folds a bounded Euler angle; it does not perform a
/// general modulo reduction. Passing a continuously growing display clock to
/// it evaluates the polynomial outside its domain and turns rotation into an
/// enormous scale/shear. Normalize ALL axes, including authored preview tilts
/// and shop quarter-turns, at the shared world-render boundary.
pub fn display_rows(angles: [f32; 3]) -> [[f32; 3]; 3] {
    let angles = angles.map(|angle| {
        // Invalid display input must not send NaN/Inf geometry to the GPU.
        if !angle.is_finite() { return 0.; }
        let angle = angle.rem_euclid(std::f32::consts::TAU);
        if angle > std::f32::consts::PI { angle - std::f32::consts::TAU } else { angle }
    });
    rc_formats::moby_light::rotation_rows(angles)
        .map(|row| [row[0], row[1], row[2]].map(f32::from_bits))
}

pub fn is_coordinate(reward: usize) -> bool { (65..=82).contains(&reward) }

/// The native Items preview spins the bolt about local X and tilts it by -1.9
/// around Y. Its shaft axis is +X toward the head. Use an exact quarter-turn
/// for a vertical world prop: head +Z, shaft down, then spin about world Z.
pub fn world_euler(class: i32, native: [f32; 3], spin: f32) -> [f32; 3] {
    if class == GOLD_BOLT_CLASS { [0., -std::f32::consts::FRAC_PI_2, spin] }
    else if class == AMMO_CRATE_CLASS { [0.,0.,spin] }
    else if class == INFOBOT_CLASS { [0., 0., spin] }
    else { native }
}

/// A coordinate uses the Infobot's native upright axes, not the replaced
/// weapon's display tilt. Other rewards retain their authored orientation.
pub fn preview_euler(reward: usize, native: [f32; 3], spin: f32) -> [f32; 3] {
    if is_coordinate(reward) { world_euler(INFOBOT_CLASS, native, spin) }
    else if matches!(reward,48|52|53) { world_euler(AMMO_CRATE_CLASS,native,spin) }
    else if reward>=96 { world_euler(GOLD_BOLT_CLASS,native,spin) }
    else { native }
}

/// Native Infobots and standalone reward props start in sequence 0. Gold
/// Bolts use sequence 1 for acquisition, not for their idle world display.
pub fn pose_sequence(class: i32, native: u8) -> u8 {
    if matches!(class,INFOBOT_CLASS|1134|287|1428|511) { 0 } else { native }
}

/// Persistent mod effects must start on a fresh background. Keep the original
/// sky clear policy when none of these world-pickup systems is enabled.
pub fn fresh_background(mixed: bool, weapons: bool, hidden_weapon: bool) -> bool {
    mixed || weapons || hidden_weapon
}

#[derive(Clone, Copy, Debug)]
pub struct Halo {
    pub centre: [f32; 3],
    pub rim: [[f32; 3]; HALO_SEGMENTS],
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
}

/// A camera-facing disc with a transparent rim. World size and angular size
/// are bounded independently; a camera crossing a pickup cannot make it fill
/// the screen. The vertical-camera case has an explicit non-degenerate basis.
pub fn halo(point: [f32; 3], camera: [f32; 3]) -> Option<Halo> {
    if !point.into_iter().chain(camera).all(f32::is_finite) { return None; }
    let d = std::array::from_fn::<_, 3, _>(|i| camera[i]-point[i]);
    let distance = d.iter().map(|v| v*v).sum::<f32>().sqrt();
    if !distance.is_finite() || !(0.25..=256.).contains(&distance) { return None; }
    let normal = d.map(|v| v/distance);
    let reference = if normal[2].abs()>0.95 { [0.,1.,0.] } else { [0.,0.,1.] };
    let right = cross(reference,normal);
    let length = right.iter().map(|v| v*v).sum::<f32>().sqrt();
    let right = right.map(|v| v/length);
    let up = cross(normal,right);
    let radius = 1.25f32.min(distance*0.12);
    let centre = std::array::from_fn(|i| point[i]+normal[i]*0.05f32.min(distance*0.1));
    let rim = std::array::from_fn(|k| {
        let angle = k as f32*std::f32::consts::TAU/HALO_SEGMENTS as f32;
        std::array::from_fn(|i| centre[i]+radius*(right[i]*angle.cos()+up[i]*angle.sin()))
    });
    Some(Halo { centre, rim })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn assert_rotation(rows: [[f32; 3]; 3]) {
        let dot = |a: [f32; 3], b: [f32; 3]| a.into_iter().zip(b).map(|(a,b)|a*b).sum::<f32>();
        for i in 0..3 {
            assert!((dot(rows[i], rows[i])-1.).abs() < 0.0001, "{rows:?}");
            for j in 0..i { assert!(dot(rows[i], rows[j]).abs() < 0.0001, "{rows:?}"); }
        }
    }
    #[test]
    fn display_rotation_cannot_extrapolate_the_native_trig_polynomial() {
        // Reproduce the original eight-second failure independently of meshes,
        // shaders, textures, fog or the renderer's instance buffer.
        let raw = rc_formats::moby_light::rotation_rows([0., 0., 2.*std::f32::consts::TAU]);
        let length = raw[0][..3].iter().map(|&v|f32::from_bits(v).powi(2)).sum::<f32>().sqrt();
        assert!(length > 3000., "native routine's unbounded-input reproduction changed");
        for tick in [0, 120, 180, 239, 240, 300, 480, 600, 1800, 36000, u64::MAX] {
            assert_rotation(display_rows([0., 0., spin_angle(tick)]));
            assert!(spin_angle(tick) >= 0. && spin_angle(tick) < std::f32::consts::TAU);
        }
    }
    #[test]
    fn every_display_axis_is_bounded_including_previews_and_shop_offsets() {
        for i in 0..720 {
            let yaw = i as f32 * std::f32::consts::TAU/240.;
            assert_rotation(display_rows([1.2, -0.7, yaw + std::f32::consts::FRAC_PI_2]));
        }
        for angle in [-1e20, -1000., -std::f32::consts::TAU, 1000., 1e20, f32::NAN, f32::INFINITY] {
            assert_rotation(display_rows([angle, angle, angle]));
        }
    }
    #[test]
    fn display_spin_repeats_without_growth_or_a_visible_turn_seam() {
        for tick in 0..240 {
            assert_eq!(spin_angle(tick), spin_angle(tick + 240*60*60*12));
            let angle = spin_angle(tick);
            let rows = display_rows([0., 0., angle]);
            assert!((rows[0][0]-angle.cos()).abs()<0.00002);
            assert!((rows[0][1]-angle.sin()).abs()<0.00002);
        }
        let a=display_rows([0., 0., spin_angle(239)]);
        let b=display_rows([0., 0., spin_angle(240)]);
        assert!((a[0][0]-b[0][0]).abs()<0.001);
        assert!((a[0][1]-b[0][1]).abs()<0.027);
    }
    #[test]
    fn infobots_stay_upright_at_every_yaw_and_never_inherit_a_weapon_pose() {
        for reward in 65..=82 {
            for spin in [-3.,-1.,0.,1.,3.] {
                assert_eq!(preview_euler(reward,[1.2,-0.7,0.4],spin),[0.,0.,spin]);
            }
        }
        for reward in [10,13,23,49,50] {
            assert_eq!(preview_euler(reward,[1.2,-0.7,0.4],2.),[1.2,-0.7,0.4]);
        }
        for class in [INFOBOT_CLASS,1134,287,1428]{assert_eq!(pose_sequence(class,1),0);}
        assert_eq!(pose_sequence(121,1),1);
    }
    #[test]
    fn real_and_proxy_gold_bolts_keep_the_head_up_while_spinning() {
        for tick in 0..240 {
            let spin=spin_angle(tick);
            let euler=world_euler(GOLD_BOLT_CLASS,[1.2,-0.7,2.],spin);
            let rows=display_rows(euler);
            // Native +X is the shaft axis toward its head; the head must always
            // be above the tail, with no sideways component or upside-down turn.
            assert!(rows[0][0].abs()<0.00002&&rows[0][1].abs()<0.00002);
            assert!((rows[0][2]-1.).abs()<0.00002);
            assert_rotation(rows);
            for reward in [96,100,169] {
                assert_eq!(preview_euler(reward,[0.8,0.4,spin],spin),euler);
            }
        }
        for reward in [48,52,53]{
            for tick in 0..240 {let spin=spin_angle(tick);assert_eq!(preview_euler(reward,[0.8,0.4,spin],spin),[0.,0.,spin]);}
        }
        assert_eq!(pose_sequence(AMMO_CRATE_CLASS,1),0);
        assert_eq!(world_euler(121,[0.2,0.3,0.4],1.),[0.2,0.3,0.4]);
    }
    #[test]
    fn halos_stay_finite_small_and_planar_even_when_viewed_from_above() {
        for camera in [[0.,0.,0.25],[0.,0.,10.],[0.,0.,-10.],[0.3,0.4,0.5],[100.,30.,20.]] {
            let halo=halo([0.;3],camera).unwrap();
            let distance=camera.iter().map(|v|v*v).sum::<f32>().sqrt();
            for p in halo.rim {
                assert!(p.into_iter().all(f32::is_finite));
                let offset=std::array::from_fn::<_,3,_>(|i|p[i]-halo.centre[i]);
                let radius=offset.iter().map(|v|v*v).sum::<f32>().sqrt();
                assert!(radius<=1.2501 && radius/distance<=0.1201);
                assert!(offset.iter().zip(camera).map(|(a,b)|a*b).sum::<f32>().abs()<0.001);
            }
        }
        for camera in [[0.;3],[0.,0.,0.1],[0.,0.,300.],[f32::NAN,0.,0.],[f32::MAX,0.,0.]] {
            assert!(halo([0.;3],camera).is_none());
        }
    }
    #[test]
    fn halo_edges_are_transparent_and_native_background_policy_is_unchanged() {
        assert_eq!(HALO_EDGE_RGBA>>24,0);
        assert!((1..=64).contains(&(HALO_CENTRE_RGBA>>24)));
        assert!(!fresh_background(false,false,false));
        for flags in [(true,false,false),(false,true,false),(false,false,true)] {
            assert!(fresh_background(flags.0,flags.1,flags.2));
        }
    }
}
