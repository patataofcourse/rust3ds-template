use ctru::error::Error as CtruError;
use std::{
    ffi::CString,
    fmt::{self, Display, Result as FmtResult},
    io::Error as IoError,
};
use toml::{de::Error as TomlDeError, ser::Error as TomlSeError};
#[cfg(feature = "audio")]
use ctru::services::ndsp::Error as NdspError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Ctru(CtruError),
    Io(IoError),
    TomlDe(TomlDeError),
    TomlSer(TomlSeError),
    #[cfg(feature = "audio")]
    Ndsp(NdspError),
    Other(String),
}

impl Display for self::Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> FmtResult {
        write!(
            fmt,
            "{}",
            match self {
                Self::Io(c) => c.to_string(),
                Self::Ctru(c) => c.to_string(),
                Self::Other(c) => c.to_string(),
                Self::TomlDe(c) => c.to_string(),
                Self::TomlSer(c) => c.to_string(),
                #[cfg(feature = "audio")]
                Self::Ndsp(c) => c.to_string(),
            }
        )
    }
}

impl std::error::Error for self::Error {}

impl From<CtruError> for self::Error {
    fn from(err: CtruError) -> Self {
        Self::Ctru(err)
    }
}

impl From<IoError> for self::Error {
    fn from(err: IoError) -> Self {
        Self::Io(err)
    }
}

impl From<TomlDeError> for self::Error {
    fn from(err: TomlDeError) -> Self {
        Self::TomlDe(err)
    }
}

impl From<TomlSeError> for self::Error {
    fn from(err: TomlSeError) -> Self {
        Self::TomlSer(err)
    }
}

#[cfg(feature = "audio")]
impl From<NdspError> for self::Error {
    fn from(value: NdspError) -> Self {
        Self::Ndsp(value)
    }
}

pub fn error_applet(msg: String) {
    use ctru_sys::{
        aptExit, errorConf, errorDisp, errorInit, errorText, CFG_LANGUAGE_EN, ERROR_TEXT_WORD_WRAP,
    };

    let msg = CString::new(msg).unwrap();

    unsafe {
        let mut error_conf: errorConf = errorConf::default();
        errorInit(&mut error_conf, ERROR_TEXT_WORD_WRAP, CFG_LANGUAGE_EN);
        errorText(&mut error_conf, msg.as_ptr());

        // Display the error
        errorDisp(&mut error_conf);
        aptExit();
    }
}
