//! Presentation policy for stationary Gold-weapon exhibits. Gameplay prices,
//! collision, ammunition and world-pickup transforms do not depend on this.

/// Drone Device's inventory definition points at the Hologuise. Show the actual
/// shield-drone actor instead, using its native horizontal pose.
pub fn model_class(item: usize, native: i32) -> i32 {
    if item == 24 {
        0x1df
    } else {
        native
    }
}

/// Keep Visibomb, Taunter and Walloper at native size. The single stationary
/// shield drone uses 150% of its native size, independent of the configured
/// gameplay drone size. Preserve the other exhibits' previous fitted size.
pub fn scale_multiplier(item: usize, native_extent: f32) -> f32 {
    match item {
        24 => 1.5,
        13 | 14 | 18 => 1.0,
        _ if native_extent.is_finite() && native_extent > 0.0 => 1.5 / native_extent,
        _ => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shield_display_is_a_drone_at_one_and_a_half_native_size() {
        assert_eq!(model_class(24, 433), 479);
        for extent in [0.25, 1.0, 4.0, f32::NAN] {
            assert_eq!(scale_multiplier(24, extent), 1.5);
        }
    }
    #[test]
    fn requested_native_weapons_keep_scale_and_other_models_keep_their_class() {
        for item in [13, 14, 18] {
            for extent in [0.1, 3.0, f32::NAN] {
                assert_eq!(scale_multiplier(item, extent), 1.0);
            }
            assert_eq!(model_class(item, 999), 999);
        }
        assert_eq!(scale_multiplier(23, 3.0), 0.5);
        assert_eq!(scale_multiplier(23, f32::NAN), 1.0);
        assert_eq!(model_class(23, 999), 999);
    }
}

/// Imported reward actors have different class scales; fit their geometry to
/// a small world pickup independently of vendor/gadget preview dimensions.
pub fn world_scale_multiplier(extent:f32)->Option<f32> {
    if !extent.is_finite() || extent<=0.0001{return None;}
    let fit=1.5/extent;(fit.is_finite()&&fit>0.).then_some(fit)
}
#[cfg(test)]mod mixed_world_tests {
    #[test]fn native_model_sizes_share_a_bounded_world_extent(){
        for extent in [0.01,0.5,1.5,100.,10000.] {
            let fit=super::world_scale_multiplier(extent).unwrap();assert!((extent*fit-1.5).abs()<0.0001);
        }
        for extent in [0.,-1.,f32::NAN,f32::INFINITY]{assert!(super::world_scale_multiplier(extent).is_none());}
    }
}
