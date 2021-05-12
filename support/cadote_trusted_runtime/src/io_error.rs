use core::result;
use sgx_tstd::boxed::Box;
use sgx_tstd;
use sgx_tstd::error;


pub fn transform_ioresult<T>(result: sgx_tstd::result::Result<T, sgx_tstd::io::Error>) -> Result<T> {
  match result {
    Ok(v) => Ok(v),
    Err(err) => {
      let new_repr = match err.raw_os_error() {
        Some(i) => Repr::Os(i),
        None => {
          match err.kind() {
            sgx_tstd::io::ErrorKind::NotFound => Repr::Simple(ErrorKind::NotFound),
            sgx_tstd::io::ErrorKind::PermissionDenied => Repr::Simple(ErrorKind::PermissionDenied),
            sgx_tstd::io::ErrorKind::ConnectionRefused => Repr::Simple(ErrorKind::ConnectionRefused),
            sgx_tstd::io::ErrorKind::ConnectionReset => Repr::Simple(ErrorKind::ConnectionReset),
            sgx_tstd::io::ErrorKind::ConnectionAborted => Repr::Simple(ErrorKind::ConnectionAborted),
            sgx_tstd::io::ErrorKind::NotConnected => Repr::Simple(ErrorKind::NotConnected),
            sgx_tstd::io::ErrorKind::AddrInUse => Repr::Simple(ErrorKind::AddrInUse),
            sgx_tstd::io::ErrorKind::AddrNotAvailable => Repr::Simple(ErrorKind::AddrNotAvailable),
            sgx_tstd::io::ErrorKind::BrokenPipe => Repr::Simple(ErrorKind::BrokenPipe),
            sgx_tstd::io::ErrorKind::AlreadyExists => Repr::Simple(ErrorKind::AlreadyExists),
            sgx_tstd::io::ErrorKind::WouldBlock => Repr::Simple(ErrorKind::WouldBlock),
            sgx_tstd::io::ErrorKind::InvalidInput => Repr::Simple(ErrorKind::InvalidInput),
            sgx_tstd::io::ErrorKind::InvalidData => Repr::Simple(ErrorKind::InvalidData),
            sgx_tstd::io::ErrorKind::TimedOut => Repr::Simple(ErrorKind::TimedOut),
            sgx_tstd::io::ErrorKind::WriteZero => Repr::Simple(ErrorKind::WriteZero),
            sgx_tstd::io::ErrorKind::Interrupted => Repr::Simple(ErrorKind::Interrupted),
            sgx_tstd::io::ErrorKind::Other => Repr::Simple(ErrorKind::Other),
            sgx_tstd::io::ErrorKind::UnexpectedEof => Repr::Simple(ErrorKind::UnexpectedEof),
            // This is why we do the whole dance
            sgx_tstd::io::ErrorKind::SgxError => Repr::Simple(ErrorKind::Other),
            _ => Repr::Simple(ErrorKind::Other)
          }
        }
      };
      Err(Error {
        repr: new_repr
      })
    }
  }
}


/*
 * The following lines are based on sgx_tstd's "io/error.rs" file, which in turn is an adjusted version of
 * Rust std's "io/error.rs". While sgx_tstd adds some SGX-specific error cases, here we remove them again. We
 * also remove all method implementations, which are irrelevant for us.
 *
 * Why all that? In order to have an ABI-compatible IO Result type between enclaves and the outside world,
 * we need to transform sgx_tstd's type to the one from regular std. However, we can't have both type in one
 * build. So we essentially replicate std's type here as target for our transformations.
 */

pub type Result<T> = result::Result<T, Error>;

#[allow(dead_code)]
pub struct Error {
  repr: Repr,
}

#[allow(dead_code)]
enum Repr {
  Os(i32),
  Simple(ErrorKind),
  Custom(Box<Custom>),
}

#[derive(Debug)]
struct Custom {
  kind: ErrorKind,
  error: Box<dyn error::Error + Send + Sync>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[allow(deprecated)]
#[non_exhaustive]
pub enum ErrorKind {
  NotFound,
  PermissionDenied,
  ConnectionRefused,
  ConnectionReset,
  ConnectionAborted,
  NotConnected,
  AddrInUse,
  AddrNotAvailable,
  BrokenPipe,
  AlreadyExists,
  WouldBlock,
  InvalidInput,
  InvalidData,
  TimedOut,
  WriteZero,
  Interrupted,
  Other,
  UnexpectedEof,
}
