use std::{
  borrow::Cow,
  env,
  fs::File,
  io::{BufRead, BufReader, LineWriter, copy, prelude::*},
  process::Command,
  str
};

use regex::Regex;

use sha2::{Digest, Sha256};

// This regex grabs all MC-generated #define statements and for each it
// captures 3 groups: name, cast, value. The "cast" group is optional.
// i.e. "#define SOMETHING   ((DWORD)0x1200L)" -> ("SOMETHING", "DWORD",
// 0x1200)
const REGEX: &str =
  r"^#define (\S+)\s+\(?(\([[:alpha:]]+\))?\s*(0x[[:xdigit:]]+)";

const INPUT_FILE: &str = "res/eventmsgs.mc";

#[cfg(not(windows))]
fn prefix_command(cmd: &str) -> Cow<str> {
  let target = env::var("TARGET").unwrap();
  let arch: &str = target.split("-").collect::<Vec<&str>>()[0];
  format!("{}-w64-mingw32-{}", arch, cmd).into()
}

#[cfg(windows)]
fn prefix_command(cmd: &str) -> Cow<'_, str> {
  cmd.into()
}

fn run_tool(cmd: &str, args: &[&str]) {
  let program = prefix_command(cmd);
  let mut command = Command::new(program.as_ref());
  match command.args(args).output() {
    Ok(out) => {
      println!("{:?}", str::from_utf8(&out.stderr).unwrap());
      println!("{:?}", str::from_utf8(&out.stdout).unwrap());
    }
    Err(err) => {
      println!("ERROR: Failed to run command: {program}, error: {err}");
    }
  }
}

fn gen_rust(generated_file: &str, header: &str, origin_hash: &str) {
  let re = Regex::new(REGEX).unwrap();

  let file_out = File::create(generated_file).unwrap();
  let mut writer = LineWriter::new(file_out);

  writer
    .write_all(
      format!("// Auto-generated from origin with SHA256 {origin_hash}.\n")
        .as_bytes()
    )
    .unwrap();

  let file_in = File::open(header).unwrap();
  for line_res in BufReader::new(file_in).lines() {
    let line = line_res.unwrap();
    if let Some(x) = re.captures(&line) {
      writer
        .write_all(
          format!("pub const {}: u32 = {};\n", &x[1], &x[3]).as_bytes()
        )
        .unwrap();
    }
  }
}

fn file_hash(f: &str) -> String {
  let mut file = File::open(f).unwrap();
  let mut hasher = Sha256::new();
  let _count = copy(&mut file, &mut hasher).unwrap();
  let formatted = format!("{:x}", hasher.finalize());
  println!("file={f}, hash={formatted}");
  formatted
}

fn file_contains(f: &str, needle: &str) -> bool {
  match File::open(f) {
    Err(_) => false,
    Ok(file) => {
      for line in BufReader::new(file).lines() {
        if line.unwrap().contains(needle) {
          println!("file={f} contains {needle}");
          return true;
        }
      }
      println!("file={f} does not contain {needle}");
      false
    }
  }
}

fn main() {
  for (key, value) in env::vars() {
    println!("Env[{key}]={value}");
  }

  let origin_hash = file_hash(INPUT_FILE);

  if cfg!(windows) {
    const GENERATED_FILE: &str = "res/eventmsgs.rs";

    if !file_contains(GENERATED_FILE, &origin_hash) {
      println!(
        "Generating {GENERATED_FILE} from {INPUT_FILE} with hash \
         {origin_hash}"
      );

      run_tool("mc.exe", &["-U", "-h", "res", "-r", "res", INPUT_FILE]);
      run_tool(
        "rc.exe",
        &["/v", "/fo", "res/eventmsgs.lib", "res/eventmsgs.rc"]
      );
      gen_rust(GENERATED_FILE, "res/eventmsgs.h", &origin_hash);
    }

    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-search=native={dir}/res");
  } else {
    let out_dir = env::var("OUT_DIR").unwrap();
    let rc = format!("{out_dir}/eventmsgs.rc");
    let lib = format!("{out_dir}/eventmsgs.lib");
    let header = format!("{out_dir}/eventmsgs.h");
    let generated_file = format!("{out_dir}/eventmsgs.rs");

    println!(
      "Generating {generated_file} from {INPUT_FILE} with hash {origin_hash}"
    );

    run_tool(
      "windmc",
      &["-U", "-h", &out_dir, "-r", &out_dir, INPUT_FILE]
    );
    run_tool("windres", &["-v", "-i", &rc, "-o", &lib]);
    gen_rust(&generated_file, &header, &origin_hash);

    println!("cargo:rustc-link-search=native={out_dir}");
  }
  println!("cargo:rustc-link-lib=dylib=eventmsgs");
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
