//! Process-local rendering isolation for the local v95 investigation.
//!
//! These options never change item grants, anchors, saves, seed generation,
//! audio, gameplay lights or user INI files. Restarting without the environment
//! variable restores normal rendering. This is not a gameplay setting.
use std::sync::OnceLock;

pub const ENV: &str = "RC_REWARD_RENDER_TEST";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    pub name: &'static str,
    pub halos: bool,
    pub models: bool,
    pub display_effects: bool,
}

impl Options {
    pub const NORMAL: Self = Self {
        name: "normal", halos: true, models: true, display_effects: true,
    };

    pub fn parse(value: &str) -> Result<Self, String> {
        let mut out = Self::NORMAL;
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "normal" => {}
            "no_halos" => { out.name = "no_halos"; out.halos = false; }
            "no_models" => { out.name = "no_models"; out.models = false; }
            "no_reward_visuals" => {
                out.name = "no_reward_visuals"; out.halos = false; out.models = false;
            }
            "no_display_effects" => {
                out.name = "no_display_effects"; out.display_effects = false;
            }
            _ => return Err(format!("Unknown {ENV} value: {value:?}")),
        }
        Ok(out)
    }
}

pub fn options() -> &'static Options {
    static OPTIONS: OnceLock<Options> = OnceLock::new();
    OPTIONS.get_or_init(|| {
        let value = std::env::var(ENV).unwrap_or_default();
        Options::parse(&value).unwrap_or_else(|error| {
            eprintln!("reward render probe: {error}; using normal rendering");
            Options::NORMAL
        })
    })
}

pub fn announce() {
    let o = options();
    println!(
        "reward render probe v95: mode={} halos={} models={} display_effects={}; world_lights={:?} sea={:?} msaa={:?}",
        o.name, o.halos, o.models, o.display_effects,
        std::env::var("RC_WORLD_LIGHTS").ok(),
        std::env::var("RC_SEA").ok(),
        std::env::var("RC_MSAA").ok(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_and_absent_values_preserve_all_rendering() {
        for value in ["", "normal", " NORMAL "] {
            assert_eq!(Options::parse(value).unwrap(), Options::NORMAL);
        }
    }

    #[test]
    fn halo_test_keeps_reward_models_and_the_native_effect_compositor() {
        let o = Options::parse("no_halos").unwrap();
        assert!(!o.halos && o.models && o.display_effects);
    }

    #[test]
    fn model_test_keeps_halos_and_the_native_effect_compositor() {
        let o = Options::parse("no_models").unwrap();
        assert!(o.halos && !o.models && o.display_effects);
    }

    #[test]
    fn combined_test_keeps_native_effects() {
        let o = Options::parse("no_reward_visuals").unwrap();
        assert!(!o.halos && !o.models && o.display_effects);
    }

    #[test]
    fn compositor_test_does_not_disable_setup_or_opaque_reward_models() {
        let o = Options::parse("no_display_effects").unwrap();
        assert!(o.halos && o.models && !o.display_effects);
    }

    #[test]
    fn invalid_value_is_rejected_instead_of_selecting_an_unrelated_test() {
        assert!(Options::parse("no_particles_but_keep_fire").is_err());
    }
}
