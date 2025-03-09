#![windows_subsystem = "windows"]
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::process::{ChildStdout, Command, Output};
use std::string::ToString;
use rouille::router;
use serde::Serialize;

const CREATE_NO_WINDOW: u32 = 0x08000000;
const PORT:i32 = 8181;
const SKETCHES_FOLDER:&str = "./sketches";
const ARDUINO_VERSION:&str = "1.0.2";

const GENERIC_OK:GenericResponse = GenericResponse{
    success:true
};

#[derive(Serialize)]
struct GenericResponse {
    success:bool
}

#[derive(Serialize)]
struct CompileResponse {
    success:bool,
    used_bytes:i32,
    used_percent:i32,
    max_bytes:i32,
    message:String,
}

#[derive(Serialize)]
struct UploadResponse {
    success:bool,
    port:String,
    message:String,
}

struct CommandOutput{
    stdout:String,
    stderr:String,
}

//2.0: use json from cli

fn main() {
    start_arduino();
}