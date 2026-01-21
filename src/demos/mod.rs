pub struct Demo {
    pub name: &'static str,
    pub bytes: &'static [u8],
}

impl Demo {
    pub const GRAVITY_NEAT: Self = Self {
        name: "Gravity neat",
        bytes: include_bytes!("gravity_neat.json_snap"),
    };

    pub const GRAVITY_DOUBLE: Self = Self {
        name: "Gravity double",
        bytes: include_bytes!("double_gravity.json_snap"),
    };

    pub const GRAVITY_TRIPLE: Self = Self {
        name: "Gravity triple",
        bytes: include_bytes!("triple_gravity.json_snap"),
    };

    pub const ALL: [&'static Self; 3] = [
        &Self::GRAVITY_NEAT,
        &Self::GRAVITY_DOUBLE,
        &Self::GRAVITY_TRIPLE,
    ];
}
