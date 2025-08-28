use super::{MAGIC, RawFtrace};
use color_eyre::eyre::Result;
use memchr::memmem::Finder;
use std::{
    io::{Error as IoError, ErrorKind as IoErrorKind},
    path::Path,
};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt as _, AsyncReadExt as _, BufReader},
};

#[derive(Debug)]
pub struct FtraceFile {
    trace_info: String,
    state: State,
    file: BufReader<File>,
}

impl FtraceFile {
    pub async fn open<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path).await?;
        Ok(Self {
            trace_info: String::new(),
            state: State::Start,
            file: BufReader::new(file),
        })
    }

    pub fn trace_info(&self) -> &str {
        &self.trace_info
    }

    async fn read_ftrace_magic(&mut self) -> Result<()> {
        let mut info_buf = Vec::new();
        let finder = Finder::new(MAGIC);
        loop {
            let buf = self.file.fill_buf().await?;
            if buf.len() < MAGIC.len() {
                break;
            }

            if let Some(i) = finder.find(buf) {
                info_buf.extend_from_slice(&buf[..i]);
                self.file.consume(i + MAGIC.len());

                self.trace_info = String::from_utf8(info_buf)?;
                return Ok(());
            }

            let len = buf.len() - MAGIC.len() + 1;
            info_buf.extend_from_slice(&buf[..len]);
            self.file.consume(len);
        }

        Err(IoError::new(IoErrorKind::UnexpectedEof, "ftrace magic not found").into())
    }

    async fn read_ftrace_entry(&mut self) -> Result<Option<RawFtrace>> {
        let mut buf = [0u8; 8];
        match self.file.read_exact(&mut buf).await {
            Ok(_) => {}
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    return Ok(None);
                } else {
                    return Err(e.into());
                }
            }
        };
        Ok(Some(RawFtrace::from(u64::from_le_bytes(buf))))
    }

    pub async fn next_entry(&mut self) -> Result<Option<RawFtrace>> {
        if self.state == State::Start {
            self.read_ftrace_magic().await?;
            self.state = State::MagicRead;
        }
        match self.state {
            State::Start => unreachable!(),
            State::MagicRead => match self.read_ftrace_entry().await? {
                Some(entry) => Ok(Some(entry)),
                None => {
                    self.state = State::Eof;
                    Ok(None)
                }
            },
            State::Eof => Ok(None),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Start,
    MagicRead,
    Eof,
}
