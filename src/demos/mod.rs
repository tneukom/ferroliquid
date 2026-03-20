pub struct Demo {
    pub name: &'static str,
    pub bytes: &'static [u8],
}

impl Demo {
    pub const GRAVITY_NEAT: Self = Self {
        name: "Gravity Neat",
        bytes: include_bytes!("gravity_neat.json_snap"),
    };

    pub const GRAVITY_TRIPLE: Self = Self {
        name: "Gravity Triple",
        bytes: include_bytes!("triple_gravity.json_snap"),
    };

    pub const NEGATIVE_GRAVITY: Self = Self {
        name: "Negative Gravity",
        bytes: include_bytes!("negative_gravity.json_snap"),
    };

    pub const RING: Self = Self {
        name: "Ring",
        bytes: include_bytes!("ring.json_snap"),
    };

    pub const BLOB: Self = Self {
        name: "Blob",
        bytes: include_bytes!("blob.json_snap"),
    };

    pub const CIRCLE: Self = Self {
        name: "Circle",
        bytes: include_bytes!("circle.json_snap"),
    };

    pub const FALLS: Self = Self {
        name: "Falls",
        bytes: include_bytes!("falls.json_snap"),
    };

    pub const SHELVES: Self = Self {
        name: "Shelves",
        bytes: include_bytes!("shelves.json_snap"),
    };

    pub const SIPHON: Self = Self {
        name: "Siphon",
        bytes: include_bytes!("siphon.json_snap"),
    };

    pub const TURBULENCE: Self = Self {
        name: "Turbulence",
        bytes: include_bytes!("turbulence.json_snap"),
    };

    pub const WAVY_CIRCLE: Self = Self {
        name: "Wavy Circle",
        bytes: include_bytes!("wavy_circle.json_snap"),
    };

    pub const ALL: [&'static Self; 11] = [
        &Self::BLOB,
        &Self::CIRCLE,
        &Self::FALLS,
        &Self::GRAVITY_NEAT,
        &Self::GRAVITY_TRIPLE,
        &Self::NEGATIVE_GRAVITY,
        &Self::RING,
        &Self::SHELVES,
        &Self::SIPHON,
        &Self::TURBULENCE,
        &Self::WAVY_CIRCLE,
    ];
}
