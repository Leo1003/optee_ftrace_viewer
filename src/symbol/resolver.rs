use crate::{symbol::region::LoadInfo, utils::FormatFn};
use addr2line::Loader;
use color_eyre::eyre::{Result, eyre};
use moka::{future::Cache, ops::compute::Op};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fmt::{Debug, Formatter},
    future::ready,
    path::{Path, PathBuf},
    sync::Arc,
};
use uuid::Uuid;

#[derive(Debug)]
pub struct CachedSymbolResolver {
    resolver: SymbolResolver,
    cache: Cache<u64, Arc<String>>,
}

impl CachedSymbolResolver {
    pub fn new(resolver: SymbolResolver) -> Self {
        Self::with_capacity(resolver, 8192)
    }

    pub fn with_capacity(resolver: SymbolResolver, capacity: u64) -> Self {
        Self {
            resolver,
            cache: Cache::builder().max_capacity(capacity).build(),
        }
    }

    pub async fn resolve_symbol(&mut self, load_info: &LoadInfo, addr: u64) -> Option<Arc<String>> {
        self.cache
            .entry(addr)
            .and_compute_with(
                |_entry| match self.resolver.resolve_symbol(load_info, addr) {
                    Some(symbol) => ready(Op::Put(Arc::new(symbol))),
                    None => ready(Op::Nop),
                },
            )
            .await
            .into_entry()
            .map(|entry| Arc::clone(entry.value()))
    }
}

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

impl Debug for SymbolResolver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymbolResolver")
            .field(
                "elf",
                &FormatFn::new(|f| {
                    // Loader does not implement Debug, so we just print the keys
                    // and a placeholder for the values.
                    f.debug_map()
                        .entries(self.elf.keys().map(|k| (k, "Loader { ... }")))
                        .finish()
                }),
            )
            .field("sources", &self.sources)
            .finish()
    }
}
