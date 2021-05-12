#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

use std::prelude::v1::*;
use std::io::BufRead;
use std::ptr;

use sgx_trts::trts;
use sgx_tstd::sgxfs;

mod io_error;


#[no_mangle]
pub fn cadote_check_arg_ptr(ptr: *const u8, size: usize) {
  //println!("Cadote runtime: Checking arg pointer {:?}, size {}", ptr, size);
  if trts::rsgx_raw_is_within_enclave(ptr, size) {
    panic!("Passing a pointer to enclave memory from outside, this is very nasty!");
  }
}

#[no_mangle]
pub fn cadote_check_return_ptr(ptr: *const u8, size: usize) {
  //println!("Cadote runtime: Checking return pointer {:?}, size {}", ptr, size);
  if trts::rsgx_raw_is_within_enclave(ptr, size) {
    panic!("Passing an allocation from within the enclave to the outside world");
  }
}

/*
 * Non-generic version of sgx_tstd::sgxfs::OpenOptions::open() to avoid monomorphization hassle.
 */
#[no_mangle]
pub fn cadote_sgxfs_openoptions_open(openopts: &sgxfs::OpenOptions, path: &str) -> sgx_tstd::io::Result<sgxfs::SgxFile> {
  openopts.open(path)
}

/*
 * Non-generic version of sgx_tstd::io::BufReader::new() to avoid monomorphization hassle.
 */
#[no_mangle]
pub fn cadote_io_bufreader_new(file: sgxfs::SgxFile) -> sgx_tstd::io::BufReader<sgxfs::SgxFile> {
  sgx_tstd::io::BufReader::new(file)
}

/*
 * Non-generic version of sgx_tstd::io::BufReader::read_line(() to avoid monomorphization hassle.
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
