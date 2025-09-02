// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use optee_teec::{Context, Operation, ParamType, Session, Uuid};
use optee_teec::{ParamNone, ParamValue};
use proto::{Command, UUID};


use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, ErrorKind};
use std::thread;

use rand::Rng;




fn hello_world(session: &mut Session) -> optee_teec::Result<()> {
    let p0 = ParamValue::new(29, 0, ParamType::ValueInout);
    let mut operation = Operation::new(0, p0, ParamNone, ParamNone, ParamNone);

    println!("original value is {:?}", operation.parameters().0.a());

    session.invoke_command(Command::IncValue as u32, &mut operation)?;
    println!("inc value is {:?}", operation.parameters().0.a());

    session.invoke_command(Command::DecValue as u32, &mut operation)?;
    println!("dec value is {:?}", operation.parameters().0.a());
    Ok(())
}
fn handle_client(mut stream: TcpStream) -> Option<[u8; 8]> {
    
    /*/
    let peer_addr = stream
        .peer_addr()
        .map_or_else(|_| "Unknown".to_string(), |addr| addr.to_string());
    */

    let mut buffer = [0; 8];

    loop {
        match stream.read(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    println!("Connection closed");
                }
                return Some(buffer);
            },
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => {
                match e.kind() {
                    ErrorKind::ConnectionReset => {
                        println!("Client connection reset")
                    },
                    _ => {
                        eprintln!("Unexpected error: {}", e);
                    }
                };
            }
        };
        return None;
    }
}


// DHKE helper function
fn power_mod(base: u64, exp: u64, modulus: u64) -> u64 {
    let mut result = 1;
    let mut base = base % modulus;
    let mut exp = exp;

    while exp > 0 {
        if exp % 2 == 1 {
            result = (result * base) % modulus;
        }
        exp >>= 1;
        base = (base * base) % modulus;
    }
    return result;
}

fn main() -> optee_teec::Result<()> {
     // Public values pre-generated
     let p: u64 = 345466091;
     let base = 124717;
     
    let mut rng = rand::rng();
    let secret_key: u64 = rng.random_range(0..p - 1);
    let public_key: u64 = power_mod(base, secret_key, p);


    // Prepare for sending of public key
    let mut stream = TcpStream::connect("127.0.0.1:9090")
        .expect("Failed to connect");

    // Send public key
    stream.write_all(format!("{}", public_key).as_bytes())
        .expect("Failed to send message");

    println!("Sent public key: {:?}", public_key);

    // Receive response
    let listener = TcpListener::bind("127.0.0.1:9091").expect("Networking error");
    println!("Server listening on port 9091");

    for stream_incoming in listener.incoming() {
        match stream_incoming {
            Ok(stream) => {
                thread::spawn(move || {
                    let received_public_key: u64 = String::from_utf8(handle_client(stream)
                            .unwrap()
                            .to_vec())
                        .unwrap()
                        .parse()
                        .unwrap();
                    println!("Received public key: {:?}", received_public_key);
                    let derived_symmetric_key: u64 = power_mod(received_public_key, secret_key, p);
                    println!("Derived symmetric key: {:?}", derived_symmetric_key);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    println!("Sent public key: {:?}", format!("{:?}", public_key).as_bytes());











    let mut ctx = Context::new()?;
    let uuid = Uuid::parse_str(UUID).unwrap();
    let mut session = ctx.open_session(uuid)?;

    hello_world(&mut session)?;

    println!("Success");
    Ok(())
}

