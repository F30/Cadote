use std::fs;
use std::io::Read;
use std::io::Write;
use std::process;

use base64;
use exitcode;
use ring::{
  rand,
  signature
};
use ring::signature::KeyPair;


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


fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() < 2 {
    exit_usage();
  }

  if args[1] == "genkey" {
    gen_key(PRIVATE_KEY_FILENAME, PUBLIC_KEY_FILENAME).expect("Key generation failed");
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
    exit_usage();
  }
}

fn exit_usage() {
  eprintln!("Usage: maxisign <genkey | sign | verify <sig-file>>");
  process::exit(exitcode::USAGE);
}

fn gen_key(privkey_filename: &str, pubkey_filename: &str) -> Result<(), Error> {
  let rng = rand::SystemRandom::new();
  let key_doc = signature::Ed25519KeyPair::generate_pkcs8(&rng)?;

  let mut private_key_file = fs::OpenOptions::new().create_new(true).write(true).open(privkey_filename)?;
  private_key_file.write_all(key_doc.as_ref())?;

  let key_pair = signature::Ed25519KeyPair::from_pkcs8(key_doc.as_ref()).unwrap();
  let mut public_key_file = fs::OpenOptions::new().create_new(true).write(true).open(pubkey_filename)?;
  public_key_file.write_all(key_pair.public_key().as_ref().as_ref())?;

  Ok(())
}

fn load_private_key(filename: &str) -> Result<signature::Ed25519KeyPair, Error> {
  let mut key_file = fs::OpenOptions::new().read(true).open(filename)?;
  let mut key_bytes = Vec::new();
  key_file.read_to_end(&mut key_bytes)?;

  let key_pair = signature::Ed25519KeyPair::from_pkcs8(&key_bytes)?;
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

  sign_message(privkey_filename, &message)
}

fn sign_message(privkey_filename: &str, message: &[u8]) -> Result<signature::Signature, Error> {
  let key_pair = load_private_key(privkey_filename)?;
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

