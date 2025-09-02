use crate::symbol::region::LoadInfo;
use addr2line::Loader;
use color_eyre::eyre::{Result, eyre};
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub struct SymbolResolver {
    elf: HashMap<Option<Uuid>, Loader>,
    sources: Vec<PathBuf>,
}

impl SymbolResolver {
    pub fn new(sources: Vec<PathBuf>) -> Self {
        Self {
            elf: HashMap::new(),
            sources,
        }
    }

    pub fn resolve_symbol(&mut self, load_info: &LoadInfo, mut addr: u64) -> Option<String> {
        let loader = self.load_elf(load_info).ok()?;
        if load_info.is_tee()
            && let Some(range) = loader.get_section_range(b".text")
        {
            addr += range.begin;
        }

        loader.find_symbol(addr).map(|s| s.to_owned())
    }

    pub fn load_elf(&mut self, load_info: &LoadInfo) -> Result<&Loader> {
        let key = match load_info {
            LoadInfo::TrustedApp(info) => Some(info.uuid),
            LoadInfo::Tee(_) => None,
        };
        if !self.elf.contains_key(&key) {
            for source in &self.sources {
                if let Some(elf_path) = Self::find_elf_in_source(source, load_info) {
                    let loader = Loader::new(elf_path).map_err(|e| eyre!("{}", e))?;
                    self.elf.insert(key, loader);
                }
            }
        }

        self.elf.get(&key).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("ELF file {} not found in sources", load_info.filename()),
            )
            .into()
        })
    }

    fn find_elf_in_source(source: &Path, load_info: &LoadInfo) -> Option<PathBuf> {
        let filename = load_info.filename();
        if source.file_name() == Some(OsStr::new(&filename)) && source.is_file() {
            return Some(source.to_path_buf());
        } else if source.is_dir() {
            let candidate = source.join(&filename);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    }
}
