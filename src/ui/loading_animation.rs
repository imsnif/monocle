use crate::ui::{bold, styled_text, GRAY_LIGHT, WHITE};

pub struct LoadingAnimation {
    scanning_indication: Option<Vec<&'static str>>,
    animation_offset: u8,
    background_color: u8,
    foreground_color: u8,
}
impl LoadingAnimation {
    pub fn new(scanning_indication: &Option<Vec<&'static str>>, animation_offset: u8) -> Self {
        LoadingAnimation {
            scanning_indication: scanning_indication.clone(),
            animation_offset,
            background_color: GRAY_LIGHT,
            foreground_color: WHITE,
        }
    }
    pub fn full_len(&self) -> usize {
        self.scanning_indication
            .as_ref()
            .and_then(|scanning_indication| scanning_indication.get(0))
            .map(|s| s.chars().count() + 3) // 3 for animation dots
            .unwrap_or(0)
    }
    pub fn mid_len(&self) -> usize {
        self.scanning_indication
            .as_ref()
            .and_then(|scanning_indication| {
                scanning_indication
                    .get(1)
                    .or_else(|| scanning_indication.get(0))
            })
            .map(|s| s.chars().count() + 3) // 3 for animation dots
            .unwrap_or(0)
    }
    pub fn short_len(&self) -> usize {
        self.scanning_indication
            .as_ref()
            .and_then(|scanning_indication| {
                scanning_indication
                    .get(2)
                    .or_else(|| scanning_indication.get(1))
                    .or_else(|| scanning_indication.get(0))
            })
            .map(|s| s.chars().count() + 3) // 3 for animation dots
            .unwrap_or(0)
    }
    pub fn render_full_length(&self) -> String {
        self.scanning_indication
            .as_ref()
            .and_then(|scanning_indication| scanning_indication.get(0))
            .map(|s| {
                styled_text(
                    self.foreground_color,
                    self.background_color,
                    &bold(&(s.to_string() + &self.animation_dots())),
                )
            })
            .unwrap_or_else(String::new)
    }
    pub fn render_mid_length(&self) -> String {
        self.scanning_indication
            .as_ref()
            .and_then(|scanning_indication| {
                scanning_indication
                    .get(1)
                    .or_else(|| scanning_indication.get(0))
            })
            .map(|s| {
                styled_text(
                    self.background_color,
                    self.foreground_color,
                    &bold(&(s.to_string() + &self.animation_dots())),
                )
            })
            .unwrap_or_else(String::new)
    }
    pub fn render_short_length(&self) -> String {
        self.scanning_indication
            .as_ref()
            .and_then(|scanning_indication| {
                scanning_indication
                    .get(2)
                    .or_else(|| scanning_indication.get(1))
                    .or_else(|| scanning_indication.get(0))
            })
            .map(|s| {
                styled_text(
                    self.background_color,
                    self.foreground_color,
                    &bold(&(s.to_string() + &self.animation_dots())),
                )
            })
            .unwrap_or_else(String::new)
    }
    fn animation_dots(&self) -> String {
        let mut to_render = String::from("");
        let dot_count = self.animation_offset % 4;
        for _ in 0..dot_count {
            to_render.push('.');
        }
        to_render
    }
}
