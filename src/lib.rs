#![no_std]

extern crate bitflags;
extern crate byteorder;
extern crate embedded_hal as hal;

use byteorder::{ByteOrder, LittleEndian};
use hal::blocking::i2c;

pub mod pdo;
pub mod rdo;
pub mod registers;

use pdo::*;
use rdo::*;
use registers::*;

pub const STUSB4500_ADDR: u8 = 0x28;

/// Address enum for STUSB4500
pub enum Address {
    /// Default address with all address pins tied low
    Default,
    /// Address determined by A1 and A0 pins. True = tied high, low = tied low.
    Strap(bool, bool),
    /// Custom address from config file etc
    Custom(u8),
}

impl Address {
    pub(crate) fn addr(&self) -> u8 {
        match self {
            Address::Default => STUSB4500_ADDR,
            Address::Strap(a1, a0) => STUSB4500_ADDR | (*a1 as u8) << 1 | (*a0 as u8) << 0,
            Address::Custom(addr) => *addr,
        }
    }
}

impl Default for Address {
    fn default() -> Self {
        Address::Default
    }
}

#[derive(Debug)]
pub enum Error<I2C> {
    I2CError(I2C),
    InvalidPdo,
    OutaRangePdo,
}

pub enum PdoChannel {
    PDO1,
    PDO2,
    PDO3,
}

pub struct STUSB4500<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C, E> STUSB4500<I2C>
where
    I2C: i2c::Write<Error = E> + i2c::Read<Error = E>,
{
    pub fn new(i2c: I2C, address: Address) -> Self {
        STUSB4500 {
            i2c,
            address: address.addr(),
        }
    }

    /// Read all interrupt registers to clear them
    pub fn clear_interrupts(&mut self) -> Result<(), Error<E>> {
        // Read all interrupt registers
        let mut _buf = [0x00; 10];
        self.i2c
            .write(self.address, &[Register::PortStatus0 as u8])
            .map_err(|err| Error::I2CError(err))?;
        self.i2c
            .read(self.address, &mut _buf)
            .map_err(|err| Error::I2CError(err))
    }

    /// Set interrupt mask
    pub fn set_alerts_mask(&mut self, alerts: AlertMask) -> Result<(), Error<E>> {
        self.write(Register::AlertStatus1Mask, alerts.bits())
    }

    /// Get active interrupt flags
    pub fn get_alerts(&mut self) -> Result<Alert, Error<E>> {
        Ok(Alert::from_bits_truncate(
            self.read(Register::AlertStatus1)?,
        ))
    }

    pub fn get_portstatus0(&mut self) -> Result<PortStatus0, Error<E>> {
        Ok(PortStatus0::from_bits_truncate(
            self.read(Register::PortStatus0)?,
        ))
    }

    pub fn get_portstatus1(&mut self) -> Result<PortStatus1, Error<E>> {
        Ok(PortStatus1::from_bits_truncate(
            self.read(Register::PortStatus1)?,
        ))
    }

    pub fn get_typec_monitoring_status0(&mut self) -> Result<TypeCMonitoringStatus0, Error<E>> {
        Ok(TypeCMonitoringStatus0::from_bits_truncate(
            self.read(Register::TypeCMonitoringStatus0)?,
        ))
    }

    pub fn get_typec_monitoring_status1(&mut self) -> Result<TypeCMonitoringStatus1, Error<E>> {
        Ok(TypeCMonitoringStatus1::from_bits_truncate(
            self.read(Register::TypeCMonitoringStatus1)?,
        ))
    }

    pub fn get_prt_status(&mut self) -> Result<PrtStatus, Error<E>> {
        Ok(PrtStatus::from_bits_truncate(
            self.read(Register::PRTStatus)?,
        ))
    }

    /// Perform a soft reset
    /// Triggers re-negotiation of PDO's.
    pub fn soft_reset(&mut self) -> Result<(), Error<E>> {
        self.write(Register::TXHeaderL, 0x0D)?;
        self.write(Register::PDCommandCtrl, 0x26)?;
        Ok(())
    }

    pub fn set_pdo(&mut self, pdo: PdoChannel, data: &Pdo) -> Result<(), Error<E>> {
        if let Pdo::Fixed { .. } = data {
            self.write_word(
                match pdo {
                    PdoChannel::PDO1 => Register::DPMSNKPDO1,
                    PdoChannel::PDO2 => Register::DPMSNKPDO2,
                    PdoChannel::PDO3 => Register::DPMSNKPDO3,
                },
                data.bits(),
            )
        } else {
            // Can only advertise fixed PDOs
            Err(Error::InvalidPdo)
        }
    }

    pub fn get_pdo(&mut self, pdo: PdoChannel) -> Result<Pdo, Error<E>> {
        Pdo::from_bits(self.read_word(match pdo {
            PdoChannel::PDO1 => Register::DPMSNKPDO1,
            PdoChannel::PDO2 => Register::DPMSNKPDO2,
            PdoChannel::PDO3 => Register::DPMSNKPDO3,
        })?)
        .ok_or(Error::InvalidPdo)
    }

    pub fn get_current_rdo(&mut self) -> Result<Rdo, Error<E>> {
        Ok(Rdo(self.read_word(Register::RDORegStatus)?))
    }

    pub fn set_num_pdo(&mut self, num: u8) -> Result<(), Error<E>> {
        match num {
            1..=3 => self.write(Register::DPMPDONumb, num),
            _ => Err(Error::OutaRangePdo),
        }
    }

    /// Unlock the NVM for reading and writing
    pub fn unlock_nvm(&mut self) -> Result<STUSB4500Nvm<I2C>, Error<E>> {
        STUSB4500Nvm::unlock(self)
    }

    // *****************************************************************
    // Raw access functions

    /// Write a byte register
    pub(crate) fn write(&mut self, register: Register, value: u8) -> Result<(), Error<E>> {
        let buf = [register as u8, value];
        self.i2c
            .write(self.address, &buf)
            .map_err(|err| Error::I2CError(err))
    }

    /// Write a word register
    pub(crate) fn write_word(&mut self, register: Register, word: u32) -> Result<(), Error<E>> {
        let mut buf = [0x00; 5];
        buf[0] = register as u8;
        LittleEndian::write_u32(&mut buf[1..], word);
        self.i2c
            .write(self.address, &buf)
            .map_err(|err| Error::I2CError(err))
    }

    /// Read a byte register
    pub(crate) fn read(&mut self, register: Register) -> Result<u8, Error<E>> {
        let mut buf = [0x00; 1];
        self.i2c
            .write(self.address, &[register as u8])
            .map_err(|err| Error::I2CError(err))?;
        self.i2c
            .read(self.address, &mut buf)
            .map_err(|err| Error::I2CError(err))?;
        Ok(buf[0])
    }

    /// Read a word register
    pub(crate) fn read_word(&mut self, register: Register) -> Result<u32, Error<E>> {
        let mut buf = [0x00; 4];
        self.i2c
            .write(self.address, &[register as u8])
            .map_err(|err| Error::I2CError(err))?;
        self.i2c
            .read(self.address, &mut buf)
            .map_err(|err| Error::I2CError(err))?;
        Ok(LittleEndian::read_u32(&buf))
    }
}

pub struct STUSB4500Nvm<'a, I2C> {
    inner: &'a mut STUSB4500<I2C>,
}

impl<I2C, E> STUSB4500Nvm<'_, I2C>
where
    I2C: i2c::Write<Error = E> + i2c::Read<Error = E>,
{
    const DEFAULT_PASSWORD: u8 = 0x47;

    pub(crate) fn unlock(inner: &mut STUSB4500<I2C>) -> Result<STUSB4500Nvm<I2C>, Error<E>> {
        inner.write(Register::NvmPassword, STUSB4500Nvm::<I2C>::DEFAULT_PASSWORD)?;
        inner.write(Register::NvmCtrl0, 0x00)?;
        inner.write(
            Register::NvmCtrl0,
            (NvmCtrl0::Power | NvmCtrl0::Enable).bits(),
        )?;

        Ok(STUSB4500Nvm { inner })
    }

    /// Lock the NVM
    pub fn lock(self) -> Result<(), Error<E>> {
        self.inner
            .write(Register::NvmCtrl0, NvmCtrl0::Enable.bits())?;
        self.inner.write(Register::NvmCtrl1, 0x00)?;
        self.inner.write(Register::NvmPassword, 0x00)
    }

    /// Read the NVM data (all five sectors)
    ///
    /// The NVM data is used to set the configuration on power-up. It can be decoded by the [GUI
    /// application][gui].
    ///
    /// [gui]: https://www.st.com/en/embedded-software/stsw-stusb002.html
    pub fn read_sectors(&mut self) -> Result<[[u8; 8]; 5], Error<E>> {
        let mut buf = [[0x00; 8]; 5];
        for (i, sector) in buf.iter_mut().enumerate() {
            *sector = self.read_sector(i as u8)?;
        }
        Ok(buf)
    }

    /// Write the NVM data (all five sectors)
    ///
    /// The NVM data is used to set the configuration on power-up. It can be generated by the [GUI
    /// application][gui].
    ///
    /// [gui]: https://www.st.com/en/embedded-software/stsw-stusb002.html
    pub fn write_sectors(&mut self, sectors: [[u8; 8]; 5]) -> Result<(), Error<E>> {
        self.erase_sectors()?;
        for (i, sector) in sectors.iter().enumerate() {
            self.write_sector(i as u8, sector)?;
        }
        Ok(())
    }

    fn issue_request(&mut self) -> Result<(), Error<E>> {
        self.issue_request_with_sector(0)
    }

    fn issue_request_with_sector(&mut self, sector: u8) -> Result<(), Error<E>> {
        self.inner.write(
            Register::NvmCtrl0,
            sector | (NvmCtrl0::Power | NvmCtrl0::Enable | NvmCtrl0::Request).bits(),
        )?;

        while NvmCtrl0::from_bits_truncate(self.inner.read(Register::NvmCtrl0)?)
            .contains(NvmCtrl0::Request)
        {}

        Ok(())
    }

    fn read_sector(&mut self, sector: u8) -> Result<[u8; 8], Error<E>> {
        self.inner
            .write(Register::NvmCtrl1, NvmCtrl1Opcode::ReadSector as u8)?;
        self.issue_request_with_sector(sector)?;

        let mut buf = [0x00; 8];
        self.inner
            .i2c
            .write(self.inner.address, &[Register::RWBuffer as u8])
            .map_err(|err| Error::I2CError(err))?;
        self.inner
            .i2c
            .read(self.inner.address, &mut buf)
            .map_err(|err| Error::I2CError(err))?;
        Ok(buf)
    }

    fn write_sector(&mut self, sector: u8, data: &[u8; 8]) -> Result<(), Error<E>> {
        let mut buf = [0x00; 9];
        buf[0] = Register::RWBuffer as u8;
        buf[1..].copy_from_slice(data);

        self.inner
            .i2c
            .write(self.inner.address, &buf)
            .map_err(|err| Error::I2CError(err))?;
        self.inner
            .write(Register::NvmCtrl1, NvmCtrl1Opcode::LoadPlr as u8)?;
        self.issue_request()?;

        self.inner
            .write(Register::NvmCtrl1, NvmCtrl1Opcode::WriteSector as u8)?;
        self.issue_request_with_sector(sector)
    }

    fn erase_sectors(&mut self) -> Result<(), Error<E>> {
        self.inner.write(
            Register::NvmCtrl1,
            NvmCtrl1Opcode::LoadSer as u8
                | (NvmCtrl1::EraseSector0
                    | NvmCtrl1::EraseSector1
                    | NvmCtrl1::EraseSector2
                    | NvmCtrl1::EraseSector3
                    | NvmCtrl1::EraseSector4)
                    .bits(),
        )?;
        self.issue_request()?;

        self.inner
            .write(Register::NvmCtrl1, NvmCtrl1Opcode::EraseSectors as u8)?;
        self.issue_request()
    }
}
