use std::fs::File;
use std::time::Instant;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, self};
use std::env::{self};

struct FileInfo
    {
        file:Option<File>,
        location:String,
        bytes:Vec<u8>,
        size:usize,
    }
impl FileInfo 
    {
        fn read_file(&mut self)
            {
                self.file = Some(File::open(&self.location).expect("Error: Open File"))
            }
        fn file_to_byte(&mut self)
            {
                match self.file 
                    {
                        Some(ref mut file_descriptor) => 
                            {
                                match File::read_to_end(file_descriptor, &mut self.bytes)
                                    {
                                        Ok(size) => 
                                            {
                                                self.size = size;
                                                println!("Done: Read -> {} bytes", size);
                                            }
                                        Err(err_val) => println!("Error: Read {}", err_val),
                                    }
                            }
                        None =>
                            {
                                println!("Error: File None");
                            }
                    }
            }
        fn write_file(&mut self)
            {
                self.file = Some(File::create(&self.location).expect("Error: Create File"));
                match self.file
                    {
                        Some(ref mut file_descriptor) =>
                            {
                                match File::write_all(file_descriptor, &mut self.bytes)
                                    {
                                        Ok(_) =>
                                            {
                                                self.size = self.bytes.len();
                                                println!("Done: Write -> {} bytes", &mut self.size);
                                            }
                                        Err(err_val) => println!("Error: Write {}", err_val),
                                    }
                            }
                        None =>
                            {
                                println!("Error: File None");
                            }
                    }
            }
    }
enum Connection
    {
        Server(String, String),
        Client(String, String),
    }
impl Connection 
    {
        fn server(self, file_info:&mut FileInfo)
            {
                print!("Server: ");
                let ip:String;
                let port:String;
                let address:String;
                match self 
                    {
                        Connection::Server(in1, in2) => 
                            {
                                ip = in1.trim_end().to_string();
                                port = in2.trim_end().to_string();
                                address = format!("{}:{}", ip, port);
                                println!("{}", address);
                            }
                        _ => return
                    }
                let socket = TcpListener::bind(address);
                for stream in socket.expect("Error: Can't Check Connections").incoming()
                    {
                        match stream
                            {
                                Ok(mut stream) =>
                                    {
                                        let stay = true;
                                        while stay 
                                            {
                                                let mut data = vec![];
                                                let start_time = Instant::now();
                                                match stream.read_to_end(&mut data)
                                                    {
                                                        Ok(res) =>
                                                            {
                                                                if res == 0
                                                                    {
                                                                        println!("Connection Closed");
                                                                        return;
                                                                    }
                                                                file_info.bytes = data;
                                                                let start_disk_time = Instant::now();
                                                                println!("Passed: Network -> {:#?}", start_disk_time.duration_since(start_time));
                                                                FileInfo::write_file(file_info);
                                                                let finish_time = Instant::now();
                                                                println!("Passed: Write -> {:#?}", finish_time.duration_since(start_disk_time));
                                                                println!("Passed: Total -> {:#?}", finish_time.duration_since(start_time));
                                                            }
                                                        Err(e) => 
                                                            {
                                                                println!("Error: Failed to Read -> {}", e);
                                                                return;
                                                            }
                                                    }
                                            }
                                    }
                                Err(e) =>
                                    {
                                        println!("Error: Can't Visit Stream -> {}", e);
                                        return;
                                    }
                            }
                    }
            }
        fn client(self, file_info:&mut FileInfo)
            {
                print!("Client: ");
                let ip:String;
                let port:String;
                let address:String;
                match self 
                    {
                        Connection::Client(in1, in2) => 
                            {
                                ip = in1.trim_end().to_string();
                                port = in2.trim_end().to_string();
                                address = format!("{}:{}", ip, port);
                                println!("{}", address);
                            }
                        _ => return
                    }
                match TcpStream::connect(address) 
                    {
                        Ok(mut socket) =>
                            {
                                let start_time = Instant::now();
                                println!("Connected");
                                FileInfo::read_file(file_info);
                                FileInfo::file_to_byte(file_info);
                                let start_network_time = Instant::now();
                                println!("Passed: Read -> {:#?}", start_network_time.duration_since(start_time));
                                socket.write_all(&file_info.bytes).unwrap();
                                let finish_time = Instant::now();
                                println!("Passed: Network -> {:#?}", finish_time.duration_since(start_network_time));
                                println!("Passed: Total -> {:#?}", finish_time.duration_since(start_time));
                            }
                        Err(e) =>
                            {
                                println!("Error: Connection -> {}", e);
                            }
                    }
                
            }
    }

fn take_string(output:String) -> String
    {
        let mut input = String::new();
        println!("{}", output);
        io::stdin().read_line(&mut input).expect("Failed to Read from Console");
        input
    }
fn take_arg() -> String
    {
        env::args().last().as_deref().unwrap_or("default").to_string()
    }



fn main() 
    {
        println!("Hello, world!");

        let bytes:Vec<u8> = vec![];
        let size:usize = 0 as usize;
        let mut data = FileInfo
            {
                file:None,
                location:take_arg(),
                bytes:bytes,
                size:size,
            };
        match &take_string("Input: Server 's', Client 'c'".to_string())[0..1]
            {
                "s" => 
                    {
                        Connection::server
                        (Connection::Server(take_string("Input: Server Stream IP Address".to_string()),
                         take_string("Input: Server Stream Port Address".to_string())),
                          &mut data);
                    },
                "c" => 
                    {
                        Connection::client
                        (Connection::Client(take_string("Input: Server IP Address to Connect".to_string()),
                         take_string("Input: Server Port Address to Connect".to_string())),
                          &mut data);
                    }
                input => 
                    {
                        println!("Error: Give Valid Input, You Gave : {}", input);
                        return;
                    }
            }
    }
