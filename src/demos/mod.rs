pub struct Demo {
    pub name: &'static str,
    pub bytes: &'static [u8],
}

impl Demo {
    pub const GRAVITY_NEAT: Self = Self {
        name: "Gravity Neat",
        bytes: include_bytes!("gravity_neat.json_snap"),
    };

    pub const GRAVITY_DOUBLE: Self = Self {
        name: "Gravity Double",
        bytes: include_bytes!("double_gravity.json_snap"),
    };

    pub const GRAVITY_TRIPLE: Self = Self {
        name: "Gravity Triple",
        bytes: include_bytes!("triple_gravity.json_snap"),
    };

    pub const NEGATIVE_GRAVITY: Self = Self {
        name: "Negative Gravity",
        bytes: include_bytes!("negative_gravity.json_snap"),
    };

    pub const CIRCULAR_SIPHON: Self = Self {
        name: "Circular Siphon",
        bytes: include_bytes!("circular_siphon.json_snap"),
    };

    pub const RING: Self = Self {
        name: "Ring",
        bytes: include_bytes!("ring.json_snap"),
    };

    pub const WHIRL: Self = Self {
        name: "Whirl",
        bytes: include_bytes!("whirl.json_snap"),
    };

    pub const FOUNTAIN: Self = Self {
        name: "Fountain",
        bytes: include_bytes!("fountain.json_snap"),
    };

    pub const TURBULENCE: Self = Self {
        name: "Turbulence",
        bytes: include_bytes!("turbulence.json_snap"),
    };

    pub const STICKY: Self = Self {
        name: "Sticky",
        bytes: include_bytes!("sticky.json_snap"),
    };

    pub const ALL: [&'static Self; 10] = [
        &Self::GRAVITY_NEAT,
        &Self::GRAVITY_DOUBLE,
        &Self::GRAVITY_TRIPLE,
        &Self::NEGATIVE_GRAVITY,
        &Self::CIRCULAR_SIPHON,
        &Self::RING,
        &Self::WHIRL,
        &Self::FOUNTAIN,
        &Self::TURBULENCE,
        &Self::STICKY,
    ];
}
