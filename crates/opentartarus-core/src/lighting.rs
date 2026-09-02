use crate::error::ErrorCode;
use crate::types::Lighting;

pub trait LightingClient {
    fn apply(&mut self, lighting: &Lighting) -> Result<(), ErrorCode>;
    fn available(&self) -> bool;
}

pub struct NullLighting;

pub struct RecordingLighting {
    pub available: bool,
    pub last: Option<Lighting>,
}

pub struct LightingChain<P, F> {
    primary: P,
    fallback: F,
}

impl<P, F> LightingChain<P, F> {
    pub fn new(primary: P, fallback: F) -> Self {
        Self { primary, fallback }
    }

    pub fn primary(&mut self) -> &mut P {
        &mut self.primary
    }

    pub fn fallback(&mut self) -> &mut F {
        &mut self.fallback
    }
}

impl<P: LightingClient, F: LightingClient> LightingClient for LightingChain<P, F> {
    fn apply(&mut self, lighting: &Lighting) -> Result<(), ErrorCode> {
        if self.primary.available() {
            return self.primary.apply(lighting);
        }
        if self.fallback.available() {
            return self.fallback.apply(lighting);
        }
        Err(ErrorCode::Lighting)
    }

    fn available(&self) -> bool {
        self.primary.available() || self.fallback.available()
    }
}

pub fn openrazer_lighting_ready(
    daemon_owns_bus: bool,
    tartarus_sysfs: bool,
    other_frontend_count: usize,
) -> bool {
    let _ = other_frontend_count;
    daemon_owns_bus || tartarus_sysfs
}

pub fn peer_frontends_make_openrazer_missing(_other_frontend_count: usize) -> bool {
    false
}

pub fn should_request_openrazer_bus_name() -> bool {
    false
}

pub fn brightness_to_razer(brightness: u8) -> u8 {
    let v = (f32::from(brightness) * 2.55).round();
    v.clamp(0.0, 255.0) as u8
}

pub fn wave_sysfs_value() -> u8 {
    1
}

impl LightingClient for NullLighting {
    fn apply(&mut self, _lighting: &Lighting) -> Result<(), ErrorCode> {
        Err(ErrorCode::Lighting)
    }

    fn available(&self) -> bool {
        false
    }
}

impl LightingClient for RecordingLighting {
    fn apply(&mut self, lighting: &Lighting) -> Result<(), ErrorCode> {
        if self.available {
            self.last = Some(lighting.clone());
            Ok(())
        } else {
            Err(ErrorCode::Lighting)
        }
    }

    fn available(&self) -> bool {
        self.available
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCode;
    use crate::types::{Lighting, LightingEffect};

    #[test]
    fn static_writes_rgb_on_mock() {
        let mut c = RecordingLighting {
            available: true,
            last: None,
        };
        let l = Lighting {
            effect: LightingEffect::Static,
            brightness: 80,
            color: Some([0, 180, 255]),
        };
        c.apply(&l).unwrap();
        assert_eq!(c.last.unwrap().color, Some([0, 180, 255]));
        assert_eq!(brightness_to_razer(80), 204);
        assert_eq!(wave_sysfs_value(), 1);
    }

    #[test]
    fn absent_returns_lighting_and_caller_can_still_apply_profile() {
        let mut c = NullLighting;
        assert!(!c.available());
        let err = c
            .apply(&Lighting {
                effect: LightingEffect::None,
                brightness: 80,
                color: None,
            })
            .unwrap_err();
        assert_eq!(err, ErrorCode::Lighting);
        assert_eq!(
            err.user_message(),
            "Lighting needs OpenRazer or OpenRGB."
        );
    }

    fn sample() -> Lighting {
        Lighting {
            effect: LightingEffect::Spectrum,
            brightness: 40,
            color: None,
        }
    }

    #[test]
    fn daemon_up_means_lighting_even_with_other_frontends() {
        const POLYCHROMATIC_AND_RAZERGENIE: usize = 2;
        assert!(openrazer_lighting_ready(true, false, POLYCHROMATIC_AND_RAZERGENIE));
        assert!(openrazer_lighting_ready(true, true, POLYCHROMATIC_AND_RAZERGENIE));
        assert!(openrazer_lighting_ready(false, true, POLYCHROMATIC_AND_RAZERGENIE));
        assert!(!openrazer_lighting_ready(false, false, POLYCHROMATIC_AND_RAZERGENIE));
        assert!(!openrazer_lighting_ready(false, false, 0));
        assert!(!peer_frontends_make_openrazer_missing(POLYCHROMATIC_AND_RAZERGENIE));
        assert!(!should_request_openrazer_bus_name());
    }

    #[test]
    fn chain_uses_openrazer_first_then_openrgb() {
        let mut chain = LightingChain::new(
            RecordingLighting {
                available: true,
                last: None,
            },
            RecordingLighting {
                available: true,
                last: None,
            },
        );
        assert!(chain.available());
        chain.apply(&sample()).unwrap();
        assert_eq!(chain.primary().last.as_ref().unwrap().effect, LightingEffect::Spectrum);
        assert!(chain.fallback().last.is_none());
    }

    #[test]
    fn chain_uses_openrgb_when_openrazer_is_down() {
        let mut chain = LightingChain::new(
            NullLighting,
            RecordingLighting {
                available: true,
                last: None,
            },
        );
        assert!(chain.available());
        chain.apply(&sample()).unwrap();
        assert_eq!(chain.fallback().last.as_ref().unwrap().brightness, 40);
    }

    #[test]
    fn chain_does_not_call_openrgb_while_openrazer_reports_available() {
        struct AvailableFail;
        impl LightingClient for AvailableFail {
            fn apply(&mut self, _lighting: &Lighting) -> Result<(), ErrorCode> {
                Err(ErrorCode::Lighting)
            }
            fn available(&self) -> bool {
                true
            }
        }
        let mut chain = LightingChain::new(
            AvailableFail,
            RecordingLighting {
                available: true,
                last: None,
            },
        );
        assert!(chain.available());
        assert_eq!(chain.apply(&sample()), Err(ErrorCode::Lighting));
        assert!(chain.fallback().last.is_none());
    }
}
