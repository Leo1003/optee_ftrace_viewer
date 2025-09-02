use crate::symbol::info::{ElfInfo, RegionData};

use super::error::SymbolError;
use bitflags::bitflags;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadInfo {
    TrustedApp(TaRegionInfo),
    Tee(TeeInfo),
}

impl LoadInfo {
    pub fn load_addr(&self) -> u64 {
        match self {
            LoadInfo::TrustedApp(info) => info.load_addr,
            LoadInfo::Tee(info) => info.load_addr,
        }
    }

    pub fn calculate_reladdr(&self, addr: u64) -> Option<u64> {
        addr.checked_sub(self.load_addr())
    }

    pub fn filename(&self) -> String {
        match self {
            LoadInfo::TrustedApp(info) => format!("{}.elf", info.uuid),
            LoadInfo::Tee(_) => "tee.elf".to_string(),
        }
    }

    pub fn is_tee(&self) -> bool {
        matches!(self, LoadInfo::Tee(_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeeInfo {
    pub load_addr: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaRegionInfo {
    pub elf_idx: usize,
    pub uuid: Uuid,
    pub load_addr: u64,
    pub va: u64,
    pub pa: u64,
    pub size: usize,
    pub flags: RegionFlags,
}

impl From<(RegionData, ElfInfo)> for TaRegionInfo {
    fn from((region, elf): (RegionData, ElfInfo)) -> Self {
        Self {
            elf_idx: elf.idx,
            uuid: elf.uuid,
            load_addr: elf.load_addr,
            va: region.va,
            pa: region.pa,
            size: region.size,
            flags: region.flags,
        }
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct RegionFlags: u32 {
        const READ = 0b0001;
        const WRITE = 0b0010;
        const EXEC = 0b0100;
        const SECURE = 0b1000;
    }
}

impl FromStr for RegionFlags {
    type Err = SymbolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut flags = RegionFlags::empty();
        for c in s.chars() {
            match c {
                'r' => flags |= RegionFlags::READ,
                'w' => flags |= RegionFlags::WRITE,
                'x' => flags |= RegionFlags::EXEC,
                's' => flags |= RegionFlags::SECURE,
                '-' => (),
                _ => return Err(SymbolError::InvalidRegionFlags),
            }
        }
        Ok(flags)
    }
}
