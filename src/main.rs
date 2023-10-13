use std::fs::{File, Metadata, self};
use std::time::Instant;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, self, BufWriter, BufReader, BufRead};
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
                                        self.open_file();
                                        self.send_file(stream);
                                    }
                                else if Metadata::is_symlink(metadata)
                                    {
                                        self.open_file();
                                        self.send_file(stream);
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
                self.forge_file();
                self.write_file(stream);
            }
        fn read_metadata(&mut self)
            {
                self.metadata = Some(fs::metadata(&self.location).expect("Error: Read Metadata"));
            }
        fn open_file(&mut self)
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
                                return;
                            }
                    }
                
            }
        fn send_file(&mut self, stream:&mut TcpStream)
            {
                let size = self.metadata.as_ref().unwrap().len();
                let mut iteration = (size/BUFFER_SIZE as u64)+1;
                self.handshake_validation(stream, size);
                println!("Size = {}", size);
                println!("Iteration = {}", iteration);
                while iteration != 0
                    {
                        iteration -= 1;
                        let mut buffer = [0u8;BUFFER_SIZE];
                        if iteration != 0
                            {
                                self.read_exact(&mut buffer);
                            }
                        else 
                            {
                                self.read_exact(&mut buffer[..(size%BUFFER_SIZE as u64) as usize]);
                            }
                        self.send_exact(&mut buffer, stream);
                        
                    }
            }
        fn handshake_validation(&mut self, stream:&mut TcpStream, size:u64)
            {
                self.send_exact(String::from(size.to_string()+"\n").as_bytes(), stream);
                match self.recv_until(stream, '\n')
                    {
                        Some(handshake_callback) =>
                            {
                                if handshake_callback == size.to_string().as_bytes().to_vec()
                                    {
                                        println!("Done: Handshake -> {}", self.location);
                                    }
                                else 
                                    {
                                        println!("Error: Handshake -> {}", self.location);
                                        println!("{:#?} ", handshake_callback);
                                        panic!()
                                    }
                            }
                        None =>
                            {
                                panic!()
                            }
                    }
            }
        fn read_exact(&mut self, buffer:&mut [u8])
            {
                match self.file.as_ref().unwrap().read_exact(buffer)
                    {
                        Ok(_) =>
                            {
                                //println!("Done: Read Bytes -> {}", self.location);
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Read Bytes -> {} | Error: {}", self.location, err_val);
                                panic!()
                            }
                    }
            }
        fn send_exact(&mut self, buffer:&[u8], stream:&mut TcpStream)
            {
                let mut stream_writer = BufWriter::new(stream.try_clone().unwrap());
                match stream_writer.write_all(buffer)
                    {
                        Ok(_) =>
                            {
                                self.size_current += buffer.len();
                                //println!("Done: Send Bytes -> {}", self.location);
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Send Bytes -> {} | Error: {}", self.location, err_val);
                                panic!();
                            }
                    }
                match stream_writer.flush()
                    {
                        Ok(_) =>
                            {
                                //println!("Done: Flush -> {}", self.location);
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Flush -> {} | Error: {}", self.location, err_val);
                                panic!()
                            }
                    }
            }
        fn recv_exact(&mut self, buffer:&mut [u8], stream:&mut TcpStream)
            {
                let mut stream_reader = BufReader::new(stream.try_clone().unwrap());
                match stream_reader.read_exact(buffer)
                    {
                        Ok(_) =>
                            {
                                self.size_current += buffer.len();
                                //println!("Done: Receive Bytes -> {}", self.location);
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Receive Bytes -> {} | Error: {}", self.location, err_val);
                                panic!();
                            }
                    }
            }
        fn recv_until(&mut self, stream:&mut TcpStream, until:char) -> Option<Vec<u8>>
            {
                let mut buffer = Vec::new();
                let mut stream_reader = BufReader::new(stream.try_clone().unwrap());
                match stream_reader.read_until(until as u8,&mut buffer)
                    {
                        Ok(_) =>
                            {
                                //println!("Done: Receive Until -> {}", self.location);
                                buffer.pop();
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Receive Until -> {} | Error: {}", self.location, err_val);
                                return None;
                            }
                    }
                return Some(buffer);
            }
        fn forge_file(&mut self)
            {
                self.file = Some(File::create(&self.location).expect("Error: Create File"));
            }
        fn handshake_recv(&mut self, stream:&mut TcpStream) -> u64
            {
                match self.recv_until(stream, '\n')
                    {
                        Some(handshake) =>
                            {
                                println!("Done: Handshake -> {}", self.location);
                                let mut handshake_terminated = handshake.clone();
                                handshake_terminated.push(b'\n');
                                self.send_exact(handshake_terminated.as_slice(), stream);
                                String::from_utf8(handshake.clone()).unwrap().parse().unwrap()
                            }
                        None =>
                            {
                                println!("Error: Handshake -> {}", self.location);
                                0
                            }
                    }
            }
        fn save_exact(&mut self, buffer:&[u8])
            {
                let mut file_writer = BufWriter::new(self.file.as_ref().unwrap());
                match file_writer.write_all(buffer)
                    {
                        Ok(_) =>
                            {
                                //println!("Done: Write -> {} | {} bytes", self.location, self.size_current);
                            }
                        Err(err_val) => 
                            {
                                println!("Error: Write -> {} | Error: {}", self.location,err_val);
                                panic!();
                            }
                    }
                match file_writer.flush()
                    {
                        Ok(_) =>
                            {
                                //println!("Done: Flush -> {}", self.location);
                            }
                        Err(err_val) => 
                            {
                                println!("Error: Flush -> {} | Error: {}", self.location,err_val);
                                panic!();
                            }
                    }
            }
        fn write_file(&mut self, stream:&mut TcpStream)
            {
                let size = self.handshake_recv(stream);
                let mut iteration:u64 = (size/BUFFER_SIZE as u64)+1;
                println!("Size = {}", size);
                println!("Iteration = {}", iteration);
                while iteration != 0
                    {
                        iteration -= 1;
                        let mut buffer = [0u8;BUFFER_SIZE];
                        self.recv_exact(&mut buffer, stream);
                        if iteration != 0
                                {
                                    self.save_exact(&buffer);
                                }
                            else 
                                {
                                    self.save_exact(&buffer[..(size%BUFFER_SIZE as u64) as usize]);
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
                                                println!("Connected");
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
                                println!("Connected");
                                let start_time = Instant::now();
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
