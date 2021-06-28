#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

use std::prelude::v1::*;
use std::collections::HashMap;
use std::io::Read;
use std::io::BufRead;
use std::ptr;

use sgx_libc;
use sgx_trts::trts;
use sgx_tstd::sgxfs;

mod io_error;


/*
 * Mapping from untrusted to enclave addresses to keep track of which pointer passed to an enclave got copied
 * where.
 * This is not concurrency-safe, but it needn't be: It's in the trusted library, so we control all calls
 * to functions accessing it.
 */
static mut MEMCPY_MAP: Option<HashMap<(*mut u8, usize), *mut u8>> = None;


#[no_mangle]
pub unsafe fn cadote_copy_to_enclave(untrusted_ptr: *const u8, size: usize) -> *mut u8 {
  if trts::rsgx_raw_is_within_enclave(untrusted_ptr, size) {
    panic!("Passing a pointer to enclave memory from outside, this is very nasty!");
  }

  if MEMCPY_MAP.is_none() {
    MEMCPY_MAP = Some(HashMap::new());
  }
  let memcpy_map = MEMCPY_MAP.as_mut().unwrap();

  let trusted_ptr = match memcpy_map.get(&(untrusted_ptr as *mut u8, size)) {
    // We don't need to identify overlapping ranges here, as they are irrelevant for immutable references
    // and there can only be a single mutable reference
    Some(p) => *p,
    None => {
      let trusted_alloc = sgx_libc::malloc(size) as *mut u8;
      std::ptr::copy_nonoverlapping(untrusted_ptr, trusted_alloc, size);
      memcpy_map.insert((untrusted_ptr as *mut u8, size), trusted_alloc);
      trusted_alloc
    }
  };

  trusted_ptr
}

#[no_mangle]
pub unsafe fn cadote_map_back_from_enclave(in_ptr_trusted: *const u8, in_size: usize) -> *mut u8 {
  let memcpy_map = MEMCPY_MAP.as_ref().unwrap();

  for ((candid_ptr_untrusted, candid_size), candid_ptr_trusted) in &*memcpy_map {
    if in_ptr_trusted >= *candid_ptr_trusted {
      let offset = in_ptr_trusted as usize - *candid_ptr_trusted as usize;
      if in_size + offset <= *candid_size {
        let result = *candid_ptr_untrusted as usize + offset;
        return result as *mut u8
      }
    }
  }

  panic!("Memory not copied from untrusted input cannot be returned from enclave!");
}

#[no_mangle]
pub unsafe fn cadote_copy_back_from_enclave() {
  let memcpy_map = MEMCPY_MAP.as_mut().unwrap();

  for ((untrusted_ptr, size), trusted_ptr) in &*memcpy_map {
    // TODO: This is very hacky, we need a nicer solution
    if *untrusted_ptr as usize >= 0x700000000000 {
      std::ptr::copy_nonoverlapping(*trusted_ptr, *untrusted_ptr, *size);
      // TODO: Releasing memory at the end of a postgate function should be safe in most cases
      // But: There will be problems with nested ECALLs
      sgx_libc::free(*trusted_ptr as *mut sgx_libc::c_void);
    }
  }
  memcpy_map.clear();
}

#[no_mangle]
pub fn cadote_check_ptr_to_enclave(ptr: *const u8, size: usize) {
  //println!("Cadote runtime: Checking arg pointer {:?}, size {}", ptr, size);
  if trts::rsgx_raw_is_within_enclave(ptr, size) {
    panic!("Passing a pointer to enclave memory from outside, this is very nasty!");
  }
}

#[no_mangle]
pub fn cadote_check_ptr_from_enclave(ptr: *const u8, size: usize) {
  //println!("Cadote runtime: Checking return pointer {:?}, size {}", ptr, size);
  if trts::rsgx_raw_is_within_enclave(ptr, size) {
    panic!("Passing an allocation from within the enclave to the outside world");
  }
}

#[no_mangle]
pub fn cadote_enclave_error_handler() -> ! {
  panic!("Unrecoverable enclave error");
}

/*
 * Non-generic version of sgx_tstd::sgxfs::OpenOptions::open() to avoid monomorphization hassle.
 */
#[no_mangle]
pub fn cadote_sgxfs_openoptions_open(openopts: &sgxfs::OpenOptions, path: &str) -> sgx_tstd::io::Result<sgxfs::SgxFile> {
  openopts.open(path)
}

/*
 * Non-generic version of sgx_tstd::sgxfs::SgxFile::read_to_end() to avoid monomorphization hassle.
 * While the method is not generic itself, it does call a generic function, which leads to the method also
 * getting monomorphized.
 */
#[no_mangle]
pub fn cadote_sgxfs_sgxfile_readtoend(file: &mut sgxfs::SgxFile, buf: &mut Vec<u8>) -> sgx_tstd::io::Result<usize> {
  file.read_to_end(buf)
}

/*
 * Non-generic version of sgx_tstd::io::BufReader::new() to avoid monomorphization hassle.
 */
#[no_mangle]
pub fn cadote_io_bufreader_new(file: sgxfs::SgxFile) -> sgx_tstd::io::BufReader<sgxfs::SgxFile> {
  sgx_tstd::io::BufReader::new(file)
}

/*
 * Non-generic version of sgx_tstd::io::BufReader::read_line() to avoid monomorphization hassle.
 */
#[no_mangle]
pub fn cadote_io_bufreader_readline(reader: &mut sgx_tstd::io::BufReader<sgxfs::SgxFile>,
                                    buf: &mut String) -> sgx_tstd::io::Result<usize> {
  reader.read_line(buf)
}

/*
 * Manual call to drop_in_place for sgx_tstd::sgxfs::SgxFile, wrapped to avoid monomorphization hassle.
 */
#[no_mangle]
pub unsafe fn cadote_drop_sgxfs_sgxfile(file: &mut sgxfs::SgxFile) {
  ptr::drop_in_place(file);
}

/*
 * Manual call to drop_in_place for sgx_tstd::io::BufReader, wrapped to avoid monomorphization hassle.
 */
#[no_mangle]
pub unsafe fn cadote_drop_io_bufreader(reader: &mut sgx_tstd::io::BufReader<sgxfs::SgxFile>) {
  ptr::drop_in_place(reader);
}

#[no_mangle]
pub fn cadote_transform_ioresult_unit(result: Result<(), sgx_tstd::io::Error>) -> io_error::Result<()> {
  io_error::transform_ioresult(result)
}

#[no_mangle]
pub fn cadote_transform_ioresult_bool(result: Result<bool, sgx_tstd::io::Error>) -> io_error::Result<bool> {
  io_error::transform_ioresult(result)
}
