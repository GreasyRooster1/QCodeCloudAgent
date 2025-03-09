#![windows_subsystem = "windows"]

mod arduino;

use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::process::{ChildStdout, Command, Output};
use std::string::ToString;
use rouille::router;
use serde::Serialize;
use crate::arduino::start_arduino;
//2.0: use json from cli

const VERSION:&str = "2.0.0";
const PORT:i32 = 8181;

//8181: main server
//8282: arduino server
//8383: py server

fn main() {

    rouille::start_server(format!("localhost:{PORT}"), move |request| {
        router!(request,
            (GET) (/status) => {
                rouille::Response::text(" ").with_additional_header("Access-Control-Allow-Origin", "*")
            },

            (GET) (/version) => {
                rouille::Response::text(VERSION).with_additional_header("Access-Control-Allow-Origin", "*")
            },

            _ => rouille::Response::empty_404()
        )
    });

}