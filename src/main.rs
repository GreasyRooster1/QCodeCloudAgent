//#![windows_subsystem = "windows"]


mod arduino;
mod python;

use std::{fs, thread};
use std::fs::File;
use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::process::{ChildStdout, Command, Output};
use std::string::ToString;
use rouille::router;
use serde::Serialize;
use crate::arduino::{start_arduino, ARDUINO_PORT};
use crate::python::{start_python, PYTHON_PORT};
//2.0: use json from cli

const VERSION:&str = "2.0.0";
const PORT:i32 = 8181;

pub struct CommandOutput{
    stdout:String,
    stderr:String,
}
#[derive(Serialize)]
pub struct GenericResponse {
    success:bool
}
pub const GENERIC_OK:GenericResponse = GenericResponse {
    success:true
};

//8181: main server
//8282: arduino server
//8383: py server
//8384: py output stream

fn main() {

    rouille::start_server(format!("localhost:{PORT}"), move |request| {
        router!(request,
            (GET) (/status) => {
                rouille::Response::text(" ").with_additional_header("Access-Control-Allow-Origin", "*")
            },

            (GET) (/version) => {
                rouille::Response::text(VERSION).with_additional_header("Access-Control-Allow-Origin", "*")
            },

            (GET) (/start/arduino) => {
                thread::spawn(start_arduino);
                rouille::Response::text(format!("{}" ,ARDUINO_PORT)).with_additional_header("Access-Control-Allow-Origin", "*")
            },
            (GET) (/start/python) => {
                thread::spawn(start_python);
                rouille::Response::text(format!("{}" ,PYTHON_PORT)).with_additional_header("Access-Control-Allow-Origin", "*")
            },

            _ => rouille::Response::empty_404().with_additional_header("Access-Control-Allow-Origin", "*")
        )
    });
}