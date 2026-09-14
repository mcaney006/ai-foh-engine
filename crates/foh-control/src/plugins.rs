/// Vendors we are willing to *name* in a mix proposal.
/// Driving them live requires the matching host (SuperRack / transform.engine)
/// and is out of scope of the open control plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vendor {
    Waves,
    FourierVst3,
    ConsoleNative,
}

#[derive(Debug, Clone)]
pub struct ApprovedPlugin {
    pub vendor: Vendor,
    pub name: String,
    /// Stable key used in proposals, e.g. "waves.c6"
    pub id: String,
    pub allowed_params: Vec<ParamSpec>,
}

#[derive(Debug, Clone)]
pub struct ParamSpec {
    pub name: String,
    pub min: f32,
    pub max: f32,
    pub unit: &'static str,
}

#[derive(Debug, Clone, Default)]
pub struct PluginMap {
    pub plugins: Vec<ApprovedPlugin>,
}

impl PluginMap {
    /// Conservative starter map. These are *names and ranges*, not a live
    /// SuperRack session.
    pub fn starter() -> Self {
        Self {
            plugins: vec![
                ApprovedPlugin {
                    vendor: Vendor::ConsoleNative,
                    name: "Channel EQ".into(),
                    id: "digico.eq".into(),
                    allowed_params: vec![
                        ParamSpec {
                            name: "gain_db".into(),
                            min: -4.0,
                            max: 4.0,
                            unit: "dB",
                        },
                        ParamSpec {
                            name: "freq_hz".into(),
                            min: 20.0,
                            max: 20_000.0,
                            unit: "Hz",
                        },
                        ParamSpec {
                            name: "q".into(),
                            min: 0.7,
                            max: 8.0,
                            unit: "",
                        },
                    ],
                },
                ApprovedPlugin {
                    vendor: Vendor::Waves,
                    name: "C6 Multiband Compressor".into(),
                    id: "waves.c6".into(),
                    allowed_params: vec![ParamSpec {
                        name: "threshold_db".into(),
                        min: -24.0,
                        max: 0.0,
                        unit: "dB",
                    }],
                },
                ApprovedPlugin {
                    vendor: Vendor::Waves,
                    name: "F6 Floating-Band Dynamic EQ".into(),
                    id: "waves.f6".into(),
                    allowed_params: vec![ParamSpec {
                        name: "gain_db".into(),
                        min: -4.0,
                        max: 4.0,
                        unit: "dB",
                    }],
                },
            ],
        }
    }

    pub fn get(&self, id: &str) -> Option<&ApprovedPlugin> {
        self.plugins.iter().find(|p| p.id == id)
    }
}
