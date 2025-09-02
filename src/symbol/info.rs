use super::error::SymbolError;
use bitflags::bitflags;
use regex::Regex;
use std::{collections::HashMap, str::FromStr, sync::LazyLock};
use uuid::Uuid;

// Format in core/kernel/user_ta.c:user_ta_dump_ftrace()
const TEE_LOAD_ADDR_RS: &str = r"TEE load address @ (?P<load_addr>0x[0-9a-f]+)";
// Format in ldelf/ta_elf.c:print_seg()
const REGION_RS: &str = r"region +[0-9]+: va (?P<va>0x[0-9a-f]+) pa (?P<pa>0x[0-9a-f]+) size (?P<size>0x[0-9a-f]+) flags (?P<flags>[rwxs-]{4}) (\[(?P<elf_idx>[0-9]+)\])?";
// Format in ldelf/ta_elf.c:ta_elf_print_mappings()
const ELF_LIST_RS: &str =
    r"\[(?P<idx>[0-9]+)\] (?P<uuid>[0-9a-f\-]+) @ (?P<load_addr>0x[0-9a-f\-]+)";
// Format in ldelf/ftrace.c:ftrace_init()
const FUNC_GRAPH_RS: &str = r"Function graph for TA: (?P<uuid>[0-9a-f\-]+) @ (?P<addr>[0-9a-f]+)";

static TEE_LOAD_ADDR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(TEE_LOAD_ADDR_RS).expect("Failed to compile load addr regex"));
static REGION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(REGION_RS).expect("Failed to compile region regex"));
static ELF_LIST_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(ELF_LIST_RS).expect("Failed to compile elf list regex"));
static FUNC_GRAPH_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(FUNC_GRAPH_RS).expect("Failed to compile function graph regex"));

#[derive(Clone, Debug)]
pub struct SymbolInfo {
    pub tee_load_addr: u64,
    pub regions: Vec<RegionData>,
    pub elf_list: HashMap<usize, ElfInfo>,
    pub title: String,
    pub ta_uuid: Uuid,
    pub ta_load_addr: u64,
}

impl FromStr for SymbolInfo {
    type Err = SymbolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines().peekable();
        // Parsing OP-TEE load address
        let tee_load_addr_line = lines.next().ok_or(SymbolError::InvalidRegionTable)?;
        let tee_load_addr_caps = TEE_LOAD_ADDR_REGEX
            .captures(tee_load_addr_line)
            .ok_or(SymbolError::InvalidSymbolInfo)?;
        let tee_load_addr_hex = tee_load_addr_caps
            .name("load_addr")
            .and_then(|m| m.as_str().strip_prefix("0x"))
            .ok_or(SymbolError::InvalidSymbolInfo)?;
        let tee_load_addr = u64::from_str_radix(tee_load_addr_hex, 16)
            .map_err(|_| SymbolError::InvalidSymbolInfo)?;

        // Parsing region table
        let mut regions = Vec::new();
        while let Some(region_line) = lines.next_if(|line| line.starts_with("region")) {
            let region_data = region_line.parse::<RegionData>()?;
            regions.push(region_data);
        }

        // Parsing ELF list
        let mut elf_list = HashMap::new();
        while let Some(elf_list_line) = lines.next_if(|line| line.trim_start().starts_with('[')) {
            let elf_info = elf_list_line.parse::<ElfInfo>()?;
            elf_list.insert(elf_info.idx, elf_info);
        }

        // Parsing function graph info
        let func_graph_line = lines.next().ok_or(SymbolError::InvalidRegionTable)?;
        let func_graph_caps = FUNC_GRAPH_REGEX
            .captures(func_graph_line)
            .ok_or(SymbolError::InvalidSymbolInfo)?;
        let uuid_str = func_graph_caps
            .name("uuid")
            .ok_or(SymbolError::InvalidSymbolInfo)?
            .as_str();
        let ta_addr_hex = func_graph_caps
            .name("addr")
            .ok_or(SymbolError::InvalidSymbolInfo)?
            .as_str();
        let ta_uuid = Uuid::parse_str(uuid_str)?;
        let ta_load_addr =
            u64::from_str_radix(ta_addr_hex, 16).map_err(|_| SymbolError::InvalidSymbolInfo)?;

        Ok(Self {
            tee_load_addr,
            regions,
            elf_list,
            title: func_graph_line.to_string(),
            ta_uuid,
            ta_load_addr,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ElfInfo {
    pub idx: usize,
    pub uuid: Uuid,
    pub load_addr: u64,
}

impl FromStr for ElfInfo {
    type Err = SymbolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let caps = ELF_LIST_REGEX
            .captures(s)
            .ok_or(SymbolError::InvalidSymbolInfo)?;

        let idx_str = caps
            .name("idx")
            .ok_or(SymbolError::InvalidSymbolInfo)?
            .as_str();
        let uuid_str = caps
            .name("uuid")
            .ok_or(SymbolError::InvalidSymbolInfo)?
            .as_str();
        let load_addr_str = caps
            .name("load_addr")
            .and_then(|m| m.as_str().strip_prefix("0x"))
            .ok_or(SymbolError::InvalidSymbolInfo)?;

        let idx = idx_str
            .parse()
            .map_err(|_| SymbolError::InvalidSymbolInfo)?;
        let uuid = Uuid::parse_str(uuid_str)?;
        let load_addr =
            u64::from_str_radix(load_addr_str, 16).map_err(|_| SymbolError::InvalidSymbolInfo)?;
        Ok(Self {
            idx,
            uuid,
            load_addr,
        })
    }
}

#[derive(Clone, Debug)]
pub struct RegionData {
    pub va: u64,
    pub pa: u64,
    pub size: usize,
    pub flags: RegionFlags,
    pub elf_idx: Option<usize>,
}

impl FromStr for RegionData {
    type Err = SymbolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let caps = REGION_REGEX
            .captures(s)
            .ok_or(SymbolError::InvalidRegionTable)?;

        let va_hex = caps
            .name("va")
            .and_then(|s| s.as_str().strip_prefix("0x"))
            .ok_or(SymbolError::InvalidRegionTable)?;
        let pa_hex = caps
            .name("pa")
            .and_then(|s| s.as_str().strip_prefix("0x"))
            .ok_or(SymbolError::InvalidRegionTable)?;
        let size_hex = caps
            .name("size")
            .and_then(|s| s.as_str().strip_prefix("0x"))
            .ok_or(SymbolError::InvalidRegionTable)?;
        let flags_str = caps
            .name("flags")
            .ok_or(SymbolError::InvalidRegionTable)?
            .as_str();

        let va = u64::from_str_radix(va_hex, 16).map_err(|_| SymbolError::InvalidRegionTable)?;
        let pa = u64::from_str_radix(pa_hex, 16).map_err(|_| SymbolError::InvalidRegionTable)?;
        let size =
            usize::from_str_radix(size_hex, 16).map_err(|_| SymbolError::InvalidRegionTable)?;
        let flags = RegionFlags::from_str(flags_str)?;
        let elf_idx = if let Some(elf_idx_str) = caps.name("elf_idx") {
            Some(
                elf_idx_str
                    .as_str()
                    .parse()
                    .map_err(|_| SymbolError::InvalidRegionTable)?,
            )
        } else {
            None
        };

        Ok(Self {
            va,
            pa,
            size,
            flags,
            elf_idx,
        })
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
