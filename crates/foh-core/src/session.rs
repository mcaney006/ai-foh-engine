use crate::audio::ChannelRole;

#[derive(Debug, Clone)]
pub struct NamedStem {
    pub name: String,
    pub role: ChannelRole,
    pub samples: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub sample_rate: u32,
    pub stems: Vec<NamedStem>,
}

impl Session {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            stems: Vec::new(),
        }
    }

    pub fn push(&mut self, name: impl Into<String>, role: ChannelRole, samples: Vec<f32>) {
        self.stems.push(NamedStem {
            name: name.into(),
            role,
            samples,
        });
    }

    pub fn stem_mut(&mut self, name: &str) -> Option<&mut NamedStem> {
        self.stems.iter_mut().find(|s| s.name == name)
    }

    pub fn names(&self) -> Vec<String> {
        self.stems.iter().map(|s| s.name.clone()).collect()
    }
}
