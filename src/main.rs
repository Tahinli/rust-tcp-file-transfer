use std::fs::File;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, self,copy};
use std::env;



fn read_file() -> File
    {
        File::open("/home/tahinli/Desktop/a.mp4").expect("Failed:Opening")
    }
fn file_to_byte(mut file:File) ->Vec<u8>
    {
        let mut buf = vec![];
        match File::read_to_end(&mut file, &mut buf)
            {
                Ok(size) => println!("Done:Reading\nFile Size = {} Bytes", size),
                Err(err_val) => println!("Failed:Reading {}", err_val),
            }
        buf
    }
fn write_file(buf:Vec<u8>)
    {
        match File::write_all(&mut File::create("/home/tahinli/Desktop/b.mp4").expect("Failed to create file"), &buf)
            {
                Ok(_) => println!("Done:Writing"),
                Err(err_val) => println!("Failed:Writing {}", err_val),
            }
    }


fn main() 
    {
        println!("Hello, world!");
        write_file(file_to_byte(read_file()));
    }
