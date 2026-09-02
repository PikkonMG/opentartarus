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
        assert_eq!(err.user_message(), "Lighting needs OpenRazer.");
    }
}
