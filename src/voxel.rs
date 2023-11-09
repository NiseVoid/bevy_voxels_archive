/// A Voxel is the data for a single voxel. It holds a material type and a value. The value is used
/// as a Signed Distance Field to create a smooth mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Voxel(u16);

impl PartialEq<u16> for Voxel {
    fn eq(&self, other: &u16) -> bool {
        self.0 == *other
    }
}

impl From<Voxel> for f32 {
    fn from(value: Voxel) -> Self {
        (Voxel::THRESHOLD_F32 - value.value() as f32) / Voxel::THRESHOLD_F32
    }
}

impl fast_surface_nets::SignedDistance for Voxel {
    fn is_negative(self) -> bool {
        self.value() > Self::THRESHOLD
    }
}

impl Voxel {
    const MATERIAL_BITS: u8 = 6;
    const VALUE_BITS: u16 = 10;

    /// The total number of materials that are possible for a voxel
    pub const MATERIALS: u8 = 1 << Self::MATERIAL_BITS;
    /// The maximum value for a voxel
    pub const MAX_MATERIAL: u8 = Self::MATERIALS - 1;

    /// The total number of values that are possible for a voxel
    pub const VALUES: u16 = 1 << Self::VALUE_BITS;
    /// The maximum value for a voxel
    pub const MAX_VALUE: u16 = Self::VALUES - 1;
    const VALUE_MASK: u16 = Self::MAX_VALUE;
    const THRESHOLD_F32: f32 = Self::MAX_VALUE as f32 / 2.;
    const THRESHOLD: u16 = Self::THRESHOLD_F32 as u16;

    /// An empty air voxel
    pub const AIR: Voxel = Voxel::new(0, 0);

    pub(crate) fn from_raw(input: u16) -> Voxel {
        Self(input)
    }

    pub(crate) fn raw(&self) -> u16 {
        self.0
    }

    /// Construct a Voxel from the specified material and value
    pub const fn new(material: u8, value: u16) -> Self {
        // TODO: Change these to debug-only branches
        if value > Self::MAX_VALUE {
            panic!("Invalid value");
        }
        if material > Self::MAX_MATERIAL {
            panic!("Invalid material");
        }
        Self(((material as u16) << Self::VALUE_BITS) + (value & Self::VALUE_MASK))
    }

    /// Get the material for this voxel
    pub fn material(&self) -> u8 {
        (self.0 >> Self::VALUE_BITS) as u8
    }

    /// Get a new Voxel with the specified sdf value
    pub fn with_value_f32(self, value: f32) -> Self {
        Self::new(
            self.material(),
            (Voxel::THRESHOLD_F32 - (value * Voxel::THRESHOLD_F32)).round() as u16,
        )
    }

    /// Get the value for this voxel
    pub fn value(&self) -> u16 {
        self.0 & Self::VALUE_MASK
    }
}

#[test]
fn test_voxel_to_sdf() {
    // Air voxels have a value of 0. In SD values this maps to a positive value
    assert_eq!(1., f32::from(Voxel::new(0, 0)));
    // Fully solid voxels have a value of Voxel::MAX_VALUE. In SD values this maps to a negative value
    assert_eq!(-1., f32::from(Voxel::new(0, Voxel::MAX_VALUE)));
}

#[test]
fn test_set_value_f32() {
    assert_eq!(0, Voxel::AIR.with_value_f32(1.).value());
    assert_eq!(Voxel::MAX_VALUE, Voxel::AIR.with_value_f32(-1.).value());
}
