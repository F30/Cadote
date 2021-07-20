#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

use std::prelude::v1::*;
use std::collections::HashMap;
use std::io::Read;
use std::io::BufRead;
use std::io::Write;
use std::ptr;

use sgx_libc;
use sgx_trts::trts;
use sgx_tstd::sgxfs;
use sgx_types;

mod io_error;


/*
 * Mapping from untrusted to enclave addresses to keep track of which pointer passed to an enclave got copied
 * where.
 * This is not concurrency-safe, but it needn't be: It's in the trusted library, so we control all calls
 * to functions accessing it.
 */
static mut COPY_TO_ENCLAVE_MAP: Option<HashMap<(*mut u8, usize), *mut u8>> = None;
/*
 * Mapping from enclave to untrusted addresses to keep track of which pointer passed to the untrusted app
 * through an OCALL got copied where.
 * For the sake of simplicity, we perform separate copies for OCALLs and do not mix up the mappings with
 * COPY_TO_ENCLAVE_MAP.
 */
static mut COPY_TO_APP_MAP: Option<HashMap<(*mut u8, usize), *mut u8>> = None;


#[no_mangle]
pub unsafe fn cadote_copy_to_enclave(untrusted_ptr: *const u8, size: usize) -> *mut u8 {
  if trts::rsgx_raw_is_within_enclave(untrusted_ptr, size) {
    panic!("Passing a pointer to enclave memory from outside, this is very nasty!");
  }

  if COPY_TO_ENCLAVE_MAP.is_none() {
    COPY_TO_ENCLAVE_MAP = Some(HashMap::new());
  }
  let copy_map = COPY_TO_ENCLAVE_MAP.as_mut().unwrap();

  let trusted_ptr = match copy_map.get(&(untrusted_ptr as *mut u8, size)) {
    // We don't need to identify overlapping ranges here, as they are irrelevant for immutable references
    // and there can only be a single mutable reference
    Some(p) => *p,
    None => {
      let trusted_alloc = sgx_libc::malloc(size) as *mut u8;
      std::ptr::copy_nonoverlapping(untrusted_ptr, trusted_alloc, size);
      copy_map.insert((untrusted_ptr as *mut u8, size), trusted_alloc);
      trusted_alloc
    }
  };

  trusted_ptr
}

#[no_mangle]
pub unsafe fn cadote_map_back_from_enclave(in_ptr_trusted: *const u8, in_size: usize) -> *mut u8 {
  let copy_map = COPY_TO_ENCLAVE_MAP.as_ref().expect("Cannot map back memory without having copied from untrusted input");

  for ((candid_ptr_untrusted, candid_size), candid_ptr_trusted) in &*copy_map {
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
  let copy_map = match COPY_TO_ENCLAVE_MAP.as_mut() {
    Some(m) => m,
    None => {
      return;
    }
  };

  for ((untrusted_ptr, size), trusted_ptr) in &*copy_map {
    // We should not copy if untrusted_ptr is read-only (from the text segment)
    // This is hard to identify, so we look at memory equality instead: The enclave code may still modify
    // the memory, but then it is OK if the program segfaults here
    if ! raw_memcmp(*trusted_ptr, *untrusted_ptr, *size) {
      std::ptr::copy_nonoverlapping(*trusted_ptr, *untrusted_ptr, *size);
    }
    // TODO: Releasing memory at the end of a postgate function should be safe in most cases
    // But: There will be problems with nested ECALLs
    sgx_libc::free(*trusted_ptr as *mut sgx_libc::c_void);
  }
  copy_map.clear();
}


#[no_mangle]
pub unsafe fn cadote_copy_to_app(trusted_ptr: *const u8, size: usize) -> *mut u8 {
  if COPY_TO_APP_MAP.is_none() {
    COPY_TO_APP_MAP = Some(HashMap::new());
  }
  let copy_map = COPY_TO_APP_MAP.as_mut().unwrap();

  let untrusted_ptr = match copy_map.get(&(trusted_ptr as *mut u8, size)) {
    Some(p) => *p,
    None => {
      let untrusted_alloc = sgx_types::sgx_ocalloc(size) as *mut u8;
      std::ptr::copy_nonoverlapping(trusted_ptr, untrusted_alloc, size);
      copy_map.insert((trusted_ptr as *mut u8, size), untrusted_alloc);
      untrusted_alloc
    }
  };

  untrusted_ptr
}

#[no_mangle]
pub unsafe fn cadote_map_back_from_app(in_ptr_untrusted: *const u8, in_size: usize) -> *mut u8 {
  let copy_map = COPY_TO_APP_MAP.as_ref().expect("Cannot map back memory without having copied trusted output");

  if trts::rsgx_raw_is_within_enclave(in_ptr_untrusted, in_size) {
    panic!("Passing a pointer to enclave memory from outside, this is very nasty!");
  }

  for ((candid_ptr_trusted, candid_size), candid_ptr_untrusted) in &*copy_map {
    if in_ptr_untrusted >= *candid_ptr_untrusted {
      let offset = in_ptr_untrusted as usize - *candid_ptr_untrusted as usize;
      if in_size + offset <= *candid_size {
        let result = *candid_ptr_trusted as usize + offset;
        return result as *mut u8
      }
    }
  }

  panic!("Memory not copied to the untrusted app cannot be returned from it!");
}

#[no_mangle]
pub unsafe fn cadote_copy_back_from_app() {
  let copy_map = match COPY_TO_APP_MAP.as_mut() {
    Some(m) => m,
    None => {
      return;
    }
  };

  for ((trusted_ptr, size), untrusted_ptr) in &*copy_map {
    // We should not copy if trusted_ptr is read-only (from the text segment), see above
    if ! raw_memcmp(*untrusted_ptr, *trusted_ptr, *size){
      std::ptr::copy_nonoverlapping(*untrusted_ptr, *trusted_ptr, *size);
    }
  }
  copy_map.clear();
}

/*
 * Compares two raw memory regions, similar to C memcmpy(). Returns true if they are equal. Implemented in
 * constant time because it's easy to do and might offer some security benefit (even if side channels are
 * generally out of our scope).
 */
unsafe fn raw_memcmp(ptr1: *const u8, ptr2: *const u8, size: usize) -> bool {
  let mut result: u8 = 0;

  for i in 0..size {
    let cmp1 = (ptr1 as usize + i) as *const u8;
    let cmp2 = (ptr2 as usize + i) as *const u8;
    result |= *cmp1 ^ *cmp2;
  }

  result == 0
}

#[no_mangle]
pub fn cadote_enclave_error_handler() -> ! {
  panic!("Unrecoverable enclave error");
}

/*
 * Wrapper around sgx_tstd::sgxfs::OpenOptions::new() to avoid name mangling issues.
 */
#[no_mangle]
pub fn cadote_sgxfs_openoptions_new() -> sgxfs::OpenOptions {
  sgxfs::OpenOptions::new()
}

/*
 * Wrapper around sgx_tstd::sgxfs::OpenOptions::read() to avoid name mangling issues.
 */
#[no_mangle]
pub fn cadote_sgxfs_openoptions_read(openopts: &mut sgxfs::OpenOptions, read: bool) -> &mut sgxfs::OpenOptions {
  openopts.read(read)
}

/*
 * Wrapper around sgx_tstd::sgxfs::OpenOptions::write() to avoid name mangling issues.
 */
#[no_mangle]
pub fn cadote_sgxfs_openoptions_write(openopts: &mut sgxfs::OpenOptions, write: bool) -> &mut sgxfs::OpenOptions {
  openopts.write(write)
}

/*
 * Wrapper around sgx_tstd::sgxfs::OpenOptions::append() to avoid name mangling issues.
 */
#[no_mangle]
pub fn cadote_sgxfs_openoptions_append(openopts: &mut sgxfs::OpenOptions, append: bool) -> &mut sgxfs::OpenOptions {
  openopts.append(append)
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
 * Wrapper around sgx_tstd::sgxfs::SgxFile::write_all() to avoid name mangling issues.
 */
#[no_mangle]
pub fn cadote_sgxfs_sgxfile_writeall(file: &mut sgxfs::SgxFile, buf: &[u8]) -> sgx_tstd::io::Result<()> {
  file.write_all(buf)
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
