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

use std::io::BufRead;
use std::io::BufReader;
use std::fs;
use std::io::Error as IOError;
use std::io::Write;

use rustyline;


static SHADOW_FILENAME: &str = "users.shadow";


// Additional guard macro for being able to use std::process::exit() in a called function
// Otherwise not required, see commit ab72d858
#[cfg(feature = "enclavization_bin")]
fn main() {
  println!("USER ADMINISTRATION TOOL");
  println!();

  // TODO: Resolve race condition by opening the file globally
  if fs::metadata(SHADOW_FILENAME).is_ok() {
    login_prompt();
  } else {
    initial_root_prompt();
  }
}

// Additional guard macro, see above
#[cfg(feature = "enclavization_bin")]
fn initial_root_prompt() {
  println!("No users available, creating root user...");

  let mut rl = rustyline::Editor::<()>::new();
  let password = get_line_or_exit(&mut rl, "Password: ");

  store_user_enclaved_ioresult_unit_("root", &password).expect("Could not write user to file");
  println!("Stored.");
  admin_loop();
}

// Additional guard macro, see above
#[cfg(feature = "enclavization_bin")]
fn login_prompt() {
  let mut rl = rustyline::Editor::<()>::new();
  let mut username: String;
  let mut password: String;

  loop {
    println!("Please authenticate!");

    username = get_line_or_exit(&mut rl, "Username: ");
    password = get_line_or_exit(&mut rl, "Password: ");

    if check_password_enclaved_ioresult_bool_(&username, &password).expect("Could not read user file") {
      break;
    }
    println!("Wrong, try again!");
    println!();
  }

  println!("🎉 Authenticated successfully!");

  if username == "root" {
    println!();
    admin_loop();
  }
}

// Additional guard macro, see above
#[cfg(feature = "enclavization_bin")]
fn admin_loop() {
  let mut rl = rustyline::Editor::<()>::new();

  loop {
    println!("Add user (a) or quit (q)?");
    let command = match rl.readline("> ") {
      // If stdin is not a TTY, readline() will include trailing newlines:
      // https://github.com/kkawakam/rustyline/issues/532
      Ok(l) => String::from(l.trim_end()),
      Err(rustyline::error::ReadlineError::Eof) => { return; },
      Err(rustyline::error::ReadlineError::Interrupted) => { return; },
      Err(e) => {
        panic!("Could not get input: {}", e);
      }
    };

    if command == "q" {
      return;
    } else if command == "a" {
      let username = get_line_or_exit(&mut rl, "Username: ");
      let password = get_line_or_exit(&mut rl, "Password: ");

      if username.is_empty() || password.is_empty() || username.contains(":") {
        println!("Invalid input, NOT stored!");
      } else {
        store_user_enclaved_ioresult_unit_(&username, &password).expect("Could not write user to file");
        println!("Stored.")
      }
      println!();
    }
  }
}

// Additional guard macro, see above
#[cfg(feature = "enclavization_bin")]
fn get_line_or_exit(rl: &mut rustyline::Editor::<()>, prompt: &str) -> String {
  let line = match rl.readline(prompt) {
    Ok(l) => String::from(l.trim_end()),
    Err(rustyline::error::ReadlineError::Eof) => String::from(""),
    Err(rustyline::error::ReadlineError::Interrupted) => {
      std::process::exit(0);
    },
    Err(e) => {
      panic!("Could not get input: {}", e);
    }
  };

  line
}

fn store_user_enclaved_ioresult_unit_(username: &str, password: &str) -> Result<(), IOError> {
  let mut shadow_file = fs::OpenOptions::new().create(true).append(true).open(SHADOW_FILENAME)?;
  // TODO: Hash
  let line = format!("{}:{}\n", username, password);
  shadow_file.write_all(line.as_bytes())?;

  Ok(())
}

fn check_password_enclaved_ioresult_bool_(username: &str, check_pwd: &str) -> Result<bool, IOError> {
  let shadow_file = fs::OpenOptions::new().read(true).open(SHADOW_FILENAME)?;
  let mut shadow_reader = BufReader::new(shadow_file);

  loop {
    let mut line = String::new();
    if shadow_reader.read_line(&mut line).unwrap() == 0 {
      break;
    }
    line = line.trim_end().to_string();

    let prefix = format!("{}:", username);
    let right_pwd = match line.strip_prefix(&prefix) {
      Some(p) => p,
      None => {
        continue;
      }
    };
    // TODO: Constant-time check
    if check_pwd == right_pwd {
      return Ok(true);
    } else {
      return Ok(false);
    }
  }

  Ok(false)
}
