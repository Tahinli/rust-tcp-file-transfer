use std::fs::{File, Metadata, self};
use std::time::Instant;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, self};
use std::env::{self};


const BUFFER_SIZE:usize = 100000;

struct FileInfo
    {
        file:Option<File>,
        location:String,
        size_current:usize,
        metadata:Option<Metadata>,
    }
impl FileInfo 
    {
        fn reading_operations(&mut self, stream:&mut TcpStream)
            {
                self.read_metadata();
                match self.metadata
                    {
                        Some(ref mut metadata) =>
                            {
                                if Metadata::is_file(metadata)
                                    {
                                        self.read_file();
                                        self.file_to_byte(stream);
                                    }
                                else if Metadata::is_symlink(metadata)
                                    {
                                        self.read_file();
                                        self.file_to_byte(stream);
                                    }
                                else 
                                    {
                                        //path recognition and creation on the other side
                                        //std:path
                                        panic!("\n\tError: Folder Transfers've not Supported yet\n")
                                    }
                            }
                        None =>
                            {
                                println!("Error: Read Metadata -> {}", self.location);
                            }
                    }
            }
        fn writing_operations(&mut self, stream:&mut TcpStream)
            {
                self.write_file(stream);
            }
        fn read_metadata(&mut self)
            {
                //use match, there is a chance to fail creation. don't pass with just some
                self.metadata = Some(fs::metadata(&self.location).expect("Error: Read Metadata"));
            }
        fn read_file(&mut self)
            {
                match File::open(&self.location)
                    {
                        Ok(file) =>
                            {
                                self.file = Some(file);
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Open File -> {} | Error: {}", self.location, err_val);
                            }
                    }
                
            }
        fn file_to_byte(&mut self, stream:&mut TcpStream)
            {
                //replace with readbuffer ?                
                let mut iteration = (self.metadata.as_ref().unwrap().len()/BUFFER_SIZE as u64)+1;
                let mut handshake = [0u8;BUFFER_SIZE];
                handshake[..self.metadata.as_ref().unwrap().len().to_string().as_bytes().len()].copy_from_slice(self.metadata.as_ref().unwrap().len().to_string().as_bytes());
                match stream.write_all(&mut handshake)
                    {
                        Ok(_) =>
                            {
                                let mut handshake_callback = [0u8;BUFFER_SIZE];
                                match stream.read_exact(&mut handshake_callback)
                                    {
                                        Ok(_) =>
                                            {
                                                println!("Done: Handshake -> {}", self.location);
                                            }
                                        Err(err_val) =>
                                            {
                                                println!("Error: Handshake Recv -> {} | Error: {}", self.location, err_val);
                                            }
                                    }
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Handshake Send -> {} | Error: {}", self.location, err_val);
                            }
                    }
                while iteration != 0
                    {
                        iteration -= 1;
                        let mut buffer = [0u8;BUFFER_SIZE];                                
                        match self.file.as_ref().as_mut().unwrap().read(&mut buffer)
                            {
                                Ok(read_size) =>
                                    {
                                        self.size_current += read_size;
                                        if iteration != 0 
                                            {
                                                match stream.write_all(&mut buffer)
                                                    {
                                                        Ok(_) =>
                                                            {
                                                                println!("Done: Send Bytes -> {} | Iteration = {}", self.location, iteration);
                                                            }
                                                        Err(err_val) =>
                                                            {
                                                                println!("Error: Send Bytes -> {} | Error: {}", self.location, err_val);
                                                                return;
                                                            }
                                                    }
                                                match stream.flush()
                                                    {
                                                        Ok(_) =>
                                                            {
                                                                println!("Done: Flush -> {}", self.location);
                                                            }
                                                        Err(err_val) =>
                                                            {
                                                                println!("Error: Flush -> {} | Error: {}", self.location, err_val);
                                                            }
                                                    }
                                            }
                                        else 
                                            {                                                
                                                let mut last_buffer:Vec<u8> = (&buffer[..(self.metadata.as_ref().unwrap().len()%BUFFER_SIZE as u64)as usize]).to_vec();                                                
                                                match stream.write_all(&mut last_buffer)
                                                    {
                                                        Ok(_) =>
                                                            {
                                                                println!("Done: Send Last Bytes -> {}", self.location);
                                                            }
                                                        Err(err_val) =>
                                                            {
                                                                println!("Error: Send Last Bytes -> {} | Error: {}", self.location, err_val);
                                                            }
                                                    }
                                            }
                                        
                                    }
                                Err(err_val) =>
                                    {
                                        println!("Error: File to Byte -> {} | Error: {}", self.location, err_val);
                                    }
                            }
                    
                    
                    }
            }
        fn write_file(&mut self, stream:&mut TcpStream)
            {
                //use match, there is a chance to fail creation. don't pass with just some
                self.file = Some(File::create(&self.location).expect("Error: Create File"));
                match self.file
                    {
                        Some(ref mut file_descriptor) =>
                            {
                                let mut handshake = [0u8;BUFFER_SIZE];
                                let mut size:u64 = 0;
                                match stream.read_exact(&mut handshake)
                                    {
                                        Ok(_) =>
                                            {
                                                todo!();
                                                size = String::from_utf8(handshake.to_vec()).unwrap().parse().unwrap();
                                                println!("Done: Handshake Recv -> {}", self.location);
                                                match stream.write_all(&mut handshake)
                                                    {
                                                        Ok(_) =>
                                                            {
                                                                println!("Done: Handshake Send -> {}", self.location);
                                                            }
                                                        Err(err_val) =>
                                                            {
                                                                println!("Error: Handshake Send -> {} | Error: {}", self.location, err_val);
                                                            }
                                                    }
                                            }
                                        Err(err_val) =>
                                            {
                                                println!("Error: Handshake Recv -> {} | Error: {}", self.location, err_val);
                                            }
                                    }
                                let mut iteration:u64 = (size%BUFFER_SIZE as u64)+1;
                                while iteration != 0
                                    {
                                        iteration -= 1;
                                        //let mut buffer: Vec<u8> = Vec::new();
                                        let mut buffer = [0u8;BUFFER_SIZE];                                
                                        match stream.read(&mut buffer)
                                            {
                                                Ok(_) =>
                                                    {
                                                        match File::write_all(file_descriptor, &mut buffer)
                                                        {
                                                            Ok(_) =>
                                                                {
                                                                    self.size_current += buffer.len();
                                                                    println!("Done: Write -> {} bytes", &mut self.size_current);
                                                                }
                                                            Err(err_val) => println!("Error: Write {}", err_val),
                                                        }
                                                    }
                                                Err(err_val) =>
                                                    {
                                                        println!("Error: Recv Bytes -> {} | Error: {}", self.location, err_val);
                                                    }
                                            }   
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
                print!("Server -> ");
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
                                        let mut stay = true;
                                        while stay 
                                            {
                                                let start_time = Instant::now();
                                                FileInfo::writing_operations(file_info, &mut stream);
                                                let finish_time = Instant::now();
                                                println!("Passed: Total -> {:#?}", finish_time.duration_since(start_time));
                                                stay = false;
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
                print!("Client -> ");
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
                        Ok(mut stream) =>
                            {
                                let start_time = Instant::now();
                                println!("Connected");
                                FileInfo::reading_operations(file_info, &mut stream);
                                let finish_time = Instant::now();
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
        io::stdin().read_line(&mut input).expect("Error: Failed to Read from Console");
        input
    }
fn take_arg() -> String
    {
        env::args().last().as_deref().unwrap_or("default").to_string()
    }



fn main() 
    {
        //DONT FORGET
        //First we should check folder structure and validation then make connection.
        println!("Hello, world!");

        let mut data = FileInfo
            {
                file:None,
                location:take_arg(),
                size_current:0 as usize,
                metadata:None,
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
