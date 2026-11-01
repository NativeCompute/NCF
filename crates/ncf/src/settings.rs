pub fn yes(v: &str) -> bool {
    matches!(v.trim(), "yes" | "true" | "1" | "on")
}

pub fn b(v: bool) -> &'static str {
    if v {
        "yes"
    } else {
        "no"
    }
}

pub fn pairs(text: &str) -> Vec<(String, String)> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .filter(|(k, _)| !k.is_empty())
        .collect()
}

#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    pub light_mode: bool,
    pub animations: bool,
    pub font_scale: f32,
    pub splash: bool,
    pub check_updates: bool,
}

impl Default for Look {
    fn default() -> Self {
        Look { light_mode: false, animations: true, font_scale: 1.0, splash: true, check_updates: true }
    }
}

impl Look {
    pub fn read(&mut self, key: &str, value: &str) -> bool {
        match key {
            "light_mode" => self.light_mode = yes(value),
            "animations" => self.animations = yes(value),
            "font_scale" => self.font_scale = value.parse::<f32>().unwrap_or(1.0).clamp(0.8, 1.4),
            "splash" => self.splash = yes(value),
            "check_updates" => self.check_updates = yes(value),
            _ => return false,
        }
        true
    }

    pub fn write(&self) -> String {
        format!(
            "light_mode = {}\nanimations = {}\nfont_scale = {:.2}\nsplash = {}\ncheck_updates = {}\n",
            b(self.light_mode),
            b(self.animations),
            self.font_scale,
            b(self.splash),
            b(self.check_updates)
        )
    }

    pub fn parse(text: &str) -> Look {
        let mut l = Look::default();
        for (k, v) in pairs(text) {
            l.read(&k, &v);
        }
        l
    }
}
