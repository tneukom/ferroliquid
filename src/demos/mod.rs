pub struct Demo {
    pub name: &'static str,
    pub bytes: &'static [u8],
    pub screenshot: egui::ImageSource<'static>,
}

impl Demo {
    pub const GRAVITY_NEAT: Self = Self {
        name: "Gravity Neat",
        bytes: include_bytes!("gravity_neat.json_snap"),
        screenshot: egui::include_image!("gravity_neat.jpg"),
    };

    pub const GRAVITY_TRIPLE: Self = Self {
        name: "Gravity Triple",
        bytes: include_bytes!("triple_gravity.json_snap"),
        screenshot: egui::include_image!("gravity_triple.jpg"),
    };

    pub const NEGATIVE_GRAVITY: Self = Self {
        name: "Negative Gravity",
        bytes: include_bytes!("negative_gravity.json_snap"),
        screenshot: egui::include_image!("negative_gravity.jpg"),
    };

    pub const RING: Self = Self {
        name: "Ring",
        bytes: include_bytes!("ring.json_snap"),
        screenshot: egui::include_image!("ring.jpg"),
    };

    pub const BLOB: Self = Self {
        name: "Blob",
        bytes: include_bytes!("blob.json_snap"),
        screenshot: egui::include_image!("blob.jpg"),
    };

    pub const CIRCLE: Self = Self {
        name: "Circle",
        bytes: include_bytes!("circle.json_snap"),
        screenshot: egui::include_image!("circle.jpg"),
    };

    pub const FALLS: Self = Self {
        name: "Falls",
        bytes: include_bytes!("falls.json_snap"),
        screenshot: egui::include_image!("falls.jpg"),
    };

    pub const SHELVES: Self = Self {
        name: "Shelves",
        bytes: include_bytes!("shelves.json_snap"),
        screenshot: egui::include_image!("shelves.jpg"),
    };

    pub const SIPHON: Self = Self {
        name: "Siphon",
        bytes: include_bytes!("siphon.json_snap"),
        screenshot: egui::include_image!("siphon.jpg"),
    };

    pub const TURBULENCE: Self = Self {
        name: "Turbulence",
        bytes: include_bytes!("turbulence.json_snap"),
        screenshot: egui::include_image!("turbulence.jpg"),
    };

    pub const WAVY_CIRCLE: Self = Self {
        name: "Wavy Circle",
        bytes: include_bytes!("wavy_circle.json_snap"),
        screenshot: egui::include_image!("wavy_circle.jpg"),
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
