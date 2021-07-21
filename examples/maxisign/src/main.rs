#![no_std]

#[cfg(feature = "enclavization_lib")]
#[macro_use]
extern crate sgx_tstd as std;

// For now, `macro_use` is required for std, see https://github.com/rust-lang/rust/issues/53977
#[cfg(feature = "enclavization_bin")]
#[macro_use]
extern crate std;

use std::prelude::v1::*;

#[cfg(feature = "enclavization_lib")]
extern crate cadote_trusted_runtime;

#[cfg(feature = "enclavization_bin")]
extern crate cadote_untrusted_runtime;

use std::fs;
use std::io::Read;
use std::io::Write;

use ring::{
  rand,
  signature
};
use ring::signature::{
  KeyPair,
  Ed25519KeyPair
};

#[cfg(feature = "enclavization_bin")]
use std::process;
#[cfg(feature = "enclavization_bin")]
use base64;
#[cfg(feature = "enclavization_bin")]
use exitcode;


static PRIVATE_KEY_FILENAME: &str = "secret.key.p8";
static PUBLIC_KEY_FILENAME: &str = "public.key.p8";

#[derive(Debug)]
enum Error {
  CryptoError,
  KeyRejected,
  IOError
}

impl From<ring::error::Unspecified> for Error {
  fn from(_: ring::error::Unspecified) -> Self { Error::CryptoError }
}

impl From<ring::error::KeyRejected> for Error {
  fn from(_: ring::error::KeyRejected) -> Self { Error::KeyRejected }
}

impl From<std::io::Error> for Error {
  fn from(_: std::io::Error) -> Self { Error::IOError }
}


// Additional guard macro for being able to use the std::process, base64 and exitcode libraries
#[cfg(feature = "enclavization_bin")]
fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() < 2 {
    eprintln!("Usage: maxisign <genkey | sign | verify <sig-file>>");
    process::exit(exitcode::USAGE);
  }

  if args[1] == "genkey" {
    gen_key_enclaved_(PRIVATE_KEY_FILENAME, PUBLIC_KEY_FILENAME).expect("Key generation failed");
  } else if args[1] == "sign" {
    let signature = sign_stdin(PRIVATE_KEY_FILENAME).expect("Signing failed");
    println!("{}", base64::encode(signature.as_ref()));
  } else if args[1] == "verify" && args.len() >= 3 {
    let mut signature_file = fs::OpenOptions::new().read(true).open(&args[2]).expect("Could not open signature");
    let mut signature_b64 = String::new();
    signature_file.read_to_string(&mut signature_b64).expect("Could not read signature");
    let signature = base64::decode(signature_b64.trim_end()).expect("Could not Base64-decode signature");

    if verify_stdin(PUBLIC_KEY_FILENAME, &signature).expect("Signature verification failed") {
      println!("Valid signature! :-)");
    } else {
      println!("Invalid signature! :-(");
    }
  } else {
    eprintln!("Usage: maxisign <genkey | sign | verify <sig-file>>");
    process::exit(exitcode::USAGE);
  }
}

fn gen_key_enclaved_(privkey_filename: &str, pubkey_filename: &str) -> Result<(), Error> {
  let rng = rand::SystemRandom::new();
  let key_doc = Ed25519KeyPair::generate_pkcs8(&rng)?;

  // TODO: Would like to use create_new(), but that is unsupported when automatically converting to
  // sgxfs::OpenOptions through Enclavization Pass
  let mut private_key_file = fs::OpenOptions::new().create(true).write(true).open(privkey_filename)?;
  private_key_file.write_all(key_doc.as_ref())?;

  let key_pair = Ed25519KeyPair::from_pkcs8(key_doc.as_ref()).unwrap();
  store_public_key(key_pair.public_key(), pubkey_filename)?;

  Ok(())
}

fn store_public_key(public_key: &<Ed25519KeyPair as KeyPair>::PublicKey, filename: &str) -> Result<(), Error> {
  let mut public_key_file = fs::OpenOptions::new().create_new(true).write(true).open(filename)?;
  public_key_file.write_all(public_key.as_ref())?;

  Ok(())
}

fn load_private_key_enclaved_(filename: &str) -> Result<Ed25519KeyPair, Error> {
  let mut key_file = fs::OpenOptions::new().read(true).open(filename)?;
  let mut key_bytes = Vec::new();
  key_file.read_to_end(&mut key_bytes)?;

  let key_pair = Ed25519KeyPair::from_pkcs8(&key_bytes)?;
  Ok(key_pair)
}

fn load_public_key(filename: &str) -> Result<signature::UnparsedPublicKey<Vec<u8>>, Error> {
  let mut key_file = fs::OpenOptions::new().read(true).open(filename)?;
  let mut key_bytes = Vec::new();
  key_file.read_to_end(&mut key_bytes)?;

  let public_key = signature::UnparsedPublicKey::new(&signature::ED25519, key_bytes);
  Ok(public_key)
}

fn sign_stdin(privkey_filename: &str) -> Result<signature::Signature, Error> {
  let mut message = Vec::new();
  std::io::stdin().read_to_end(&mut message)?;

  sign_message_enclaved_(privkey_filename, &message)
}

fn sign_message_enclaved_(privkey_filename: &str, message: &[u8]) -> Result<signature::Signature, Error> {
  let key_pair = load_private_key_enclaved_(privkey_filename)?;
  let signature = key_pair.sign(message);

  Ok(signature)
}

fn verify_stdin(pubkey_filename: &str, signature: &[u8]) -> Result<bool, Error> {
  let mut message = Vec::new();
  std::io::stdin().read_to_end(&mut message)?;

  verify_message(pubkey_filename, signature, &message)
}

fn verify_message(pubkey_filename: &str, signature: &[u8], message: &[u8]) -> Result<bool, Error> {
  let public_key = load_public_key(pubkey_filename)?;

  match public_key.verify(message, signature) {
    Ok(()) => Ok(true),
    Err(_) => Ok(false)
  }
}

