use bitfield::bitfield;

#[derive(Debug, PartialOrd, PartialEq)]
pub enum FastSwapSupport {
    NotSupported = 0,
    DefaultUsb = 1,
    _1A5_5V = 2,
    _3A0_5V = 3,
}

impl Default for FastSwapSupport {
    fn default() -> Self {
        FastSwapSupport::NotSupported
    }
}

impl Into<u32> for FastSwapSupport {
    fn into(self) -> u32 {
        match self {
            FastSwapSupport::NotSupported => 0,
            FastSwapSupport::DefaultUsb => 1,
            FastSwapSupport::_1A5_5V => 2,
            FastSwapSupport::_3A0_5V => 3,
        }
    }
}

const PDO_SNK_FIXED: u32 = 0x0 << 30;

bitfield! {
    pub struct FixedPdo(u32);
    impl Debug;
    // The fields default to u16
    pub fixed, _: 31, 30;
    pub dual_role_power, set_dual_role_power: 29;
    pub higher_capability, set_higher_capability: 28;
    pub unconstrained_power, set_unconstrained_power: 27;
    pub usb_communications_capable, set_usb_communications_capable: 26;
    pub dual_role_data, set_dual_role_data: 25;
    pub fast_role_swap, set_fast_role_swap: 24, 23;
    pub reserved, _: 22, 20;
    pub voltage, set_voltage: 19, 10;
    pub current, set_current: 9, 0;
}

impl Default for FixedPdo {
    fn default() -> Self {
        Self(PDO_SNK_FIXED)
    }
}

impl FixedPdo {
    pub fn new(voltage: u16, current: u16) -> Self {
        let mut pdo: Self = Default::default();
        pdo.set_voltage(voltage as u32);
        pdo.set_current(current as u32);
        pdo
    }
}

const PDO_SNK_VARIABLE: u32 = 0x1 << 30;
bitfield! {
    pub struct VariablePdo(u32);
    impl Debug;
    // The fields default to u16
    pub variable, _: 31, 30;
    pub max_voltage, set_max_voltage: 29, 20;
    pub min_voltage, set_min_voltage: 19, 10;
    pub current, set_current: 9, 0;
}

impl Default for VariablePdo {
    fn default() -> Self {
        Self(PDO_SNK_VARIABLE)
    }
}

const PDO_SNK_BATTERY: u32 = 0x2 << 30;
bitfield! {
    pub struct BatteryPdo(u32);
    impl Debug;
    // The fields default to u16
    pub battery, _: 31, 30;
    pub max_voltage, set_max_voltage: 29, 20;
    pub min_voltage, set_min_voltage: 19, 10;
    pub power, set_power: 9, 0;
}
impl Default for BatteryPdo {
    fn default() -> Self {
        Self(PDO_SNK_BATTERY)
    }
}

pub enum Pdo {
    Fixed(FixedPdo),
    Variable(VariablePdo),
    Battery(BatteryPdo),
}
impl Pdo {
    pub fn new_fixed(voltage: u16, current: u16) -> Self {
        Pdo::Fixed(FixedPdo::new(voltage, current))
    }

    pub fn dual_role_power(&mut self, dual: bool) -> &mut Self {
        if let Pdo::Fixed(x) = self {
            x.set_dual_role_power(dual);
        }
        self
    }

    pub fn dual_role_data(&mut self, dual: bool) -> &mut Self {
        if let Pdo::Fixed(x) = self {
            x.set_dual_role_data(dual);
        }
        self
    }

    pub fn usb_communications_capable(&mut self, capable: bool) -> &mut Self {
        if let Pdo::Fixed(x) = self {
            x.set_usb_communications_capable(capable);
        }
        self
    }

    pub fn higher_capability(&mut self, capable: bool) -> &mut Self {
        if let Pdo::Fixed(x) = self {
            x.set_higher_capability(capable);
        }
        self
    }

    pub fn unconstrained_power(&mut self, unconstrained: bool) -> &mut Self {
        if let Pdo::Fixed(x) = self {
            x.set_unconstrained_power(unconstrained);
        }
        self
    }

    pub(crate) fn bits(&self) -> u32 {
        match self {
            Pdo::Fixed(a) => a.0,
            Pdo::Variable(a) => a.0,
            Pdo::Battery(a) => a.0,
        }
    }

    pub fn from_bits(bits: u32) -> Option<Self> {
        match bits & 0xC000_0000 {
            PDO_SNK_FIXED => Some(Pdo::Fixed(FixedPdo(bits))),
            PDO_SNK_VARIABLE => Some(Pdo::Variable(VariablePdo(bits))),
            PDO_SNK_BATTERY => Some(Pdo::Battery(BatteryPdo(bits))),
            _ => None,
        }
    }
}
