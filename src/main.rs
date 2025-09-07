use std::fs::{File, Metadata, self};
use std::time::Instant;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, self, BufWriter, BufReader, BufRead};
use std::env::{self};


const BUFFER_SIZE:u64 = 100000;
struct FileInfo
    {
        file:Option<File>,
        location:String,
        size_current:usize,
        metadata:Option<Metadata>,
    }
impl FileInfo 
    {
        fn reading_operations(&mut self, stream:&mut TcpStream, debug_mode:&bool)
            {
                self.read_metadata();
                match self.metadata
                    {
                        Some(ref mut metadata) =>
                            {
                                if Metadata::is_file(metadata)
                                    {
                                        self.open_file();
                                        self.send_file(stream, debug_mode);
                                    }
                                else if Metadata::is_symlink(metadata)
                                    {
                                        self.open_file();
                                        self.send_file(stream, debug_mode);
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
        fn writing_operations(&mut self, stream:&mut TcpStream, debug_mode:&bool)
            {
                self.forge_file();
                self.write_file(stream, debug_mode);
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
        fn send_file(&mut self, stream:&mut TcpStream, debug_mode:&bool)
            {
                let size = self.metadata.as_ref().unwrap().len();
                let mut iteration = (size/BUFFER_SIZE)+1;
                let total_iteration = iteration;
                self.handshake_validation(stream, size, debug_mode);
                println!("Size = {}", size);
                println!("Iteration = {}", iteration);
                while iteration != 0
                    {
                        iteration -= 1;
                        let mut buffer = [0u8;BUFFER_SIZE as usize];
                        if iteration != 0
                            {
                                self.read_exact(&mut buffer, debug_mode);
                            }
                        else 
                            {
                                self.read_exact(&mut buffer[..(size%BUFFER_SIZE) as usize], debug_mode);
                            }
                        if *debug_mode
                            {
                                println!("Read Data = {:#?}", buffer);
                            }
                        self.send_exact(&mut buffer, stream, debug_mode);
                        println!("%{}", 100 as f64 -((iteration as f64/total_iteration as f64)*100 as f64));
                    }
            }
        fn handshake_validation(&mut self, stream:&mut TcpStream, size:u64, debug_mode:&bool)
            {
                self.send_exact(String::from(size.to_string()+"\n").as_bytes(), stream, debug_mode);
                match self.recv_until(stream, '\n', debug_mode)
                    {
                        Some(handshake_callback) =>
                            {
                                if handshake_callback == size.to_string().as_bytes().to_vec()
                                    {
                                        println!("Done: Handshake -> {}", self.location);
                                        if *debug_mode
                                            {
                                                println!("{:#?} ", handshake_callback);
                                            }
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
        fn read_exact(&mut self, buffer:&mut [u8], debug_mode:&bool)
            {
                match self.file.as_ref().unwrap().read_exact(buffer)
                    {
                        Ok(_) =>
                            {
                                if *debug_mode
                                    {
                                        println!("Done: Read Bytes -> {}", self.location);
                                        println!("{:#?}", buffer);
                                    }
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Read Bytes -> {} | Error: {}", self.location, err_val);
                                panic!()
                            }
                    }
            }
        fn send_exact(&mut self, buffer:&[u8], stream:&mut TcpStream, debug_mode:&bool)
            {
                let mut stream_writer = BufWriter::new(stream.try_clone().unwrap());
                match stream_writer.write_all(buffer)
                    {
                        Ok(_) =>
                            {
                                self.size_current += buffer.len();
                                if *debug_mode
                                    {
                                        println!("Done: Send Bytes -> {}", self.location);
                                        println!("{:#?}", buffer);
                                    }
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
                                if *debug_mode
                                    {
                                        println!("Done: Flush -> {}", self.location);
                                    }
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Flush -> {} | Error: {}", self.location, err_val);
                                panic!()
                            }
                    }
            }
        fn recv_exact(&mut self, buffer:&mut [u8], stream:&mut TcpStream, debug_mode:&bool)
            {
                match stream.read_exact(buffer)
                    {
                        Ok(_) =>
                            {
                                self.size_current += buffer.len();
                                if *debug_mode
                                    {
                                        println!("Done: Receive Bytes -> {}", self.location);
                                        println!("{:#?}", buffer);
                                    }
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Receive Bytes -> {} | Error: {}", self.location, err_val);
                                panic!();
                            }
                    }
            }
        fn recv_until(&mut self, stream:&mut TcpStream, until:char, debug_mode:&bool) -> Option<Vec<u8>>
            {
                let mut buffer = Vec::new();
                let mut stream_reader = BufReader::new(stream.try_clone().unwrap());
                match stream_reader.read_until(until as u8,&mut buffer)
                    {
                        Ok(_) =>
                            {
                                if *debug_mode
                                    {
                                        println!("Done: Receive Until -> {}", self.location);
                                        println!("{:#?}", buffer);
                                    }
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
        fn handshake_recv(&mut self, stream:&mut TcpStream, debug_mode:&bool) -> u64
            {
                match self.recv_until(stream, '\n', debug_mode)
                    {
                        Some(mut handshake) =>
                            {
                                println!("Done: Handshake -> {}", self.location);
                                if *debug_mode
                                    {
                                        println!("{:#?} ", handshake);
                                    }
                                let size = String::from_utf8(handshake.clone()).unwrap().parse().unwrap();
                                handshake.push(b'\n');
                                self.send_exact(&handshake.as_slice(), stream, debug_mode);
                                size
                            }
                        None =>
                            {
                                println!("Error: Handshake -> {}", self.location);
                                0
                            }
                    }
            }
        fn save_exact(&mut self, buffer:&[u8], debug_mode:&bool)
            {
                let mut file_writer = BufWriter::new(self.file.as_ref().unwrap());
                match file_writer.write_all(buffer)
                    {
                        Ok(_) =>
                            {
                                if *debug_mode
                                    {
                                        println!("Done: Write -> {} | {} bytes", self.location, self.size_current);
                                        println!("{:#?}", buffer);
                                    }
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
                                if *debug_mode
                                    {
                                        println!("Done: Flush -> {}", self.location);
                                    }
                            }
                        Err(err_val) => 
                            {
                                println!("Error: Flush -> {} | Error: {}", self.location,err_val);
                                panic!();
                            }
                    }
            }
        fn write_file(&mut self, stream:&mut TcpStream, debug_mode:&bool)
            {
                let size = self.handshake_recv(stream, debug_mode);
                let mut iteration:u64 = (size/BUFFER_SIZE)+1;
                let total_iteration = iteration;
                println!("Size = {}", size);
                println!("Iteration = {}", iteration);
                while iteration != 0
                    {
                        iteration -= 1;
                        let mut buffer = [0u8;BUFFER_SIZE as usize];
                        self.recv_exact(&mut buffer, stream, debug_mode);
                        if iteration != 0
                                {
                                    self.save_exact(&buffer, debug_mode);
                                }
                            else 
                                {
                                    self.save_exact(&buffer[..(size%BUFFER_SIZE) as usize], debug_mode);
                                }
                        println!("%{}", 100 as f64 -((iteration as f64/total_iteration as f64)*100 as f64));
                    }            
            }
    }
enum DebugMode
    {
        On,
        Off
    }
impl DebugMode {
    fn debug_mode(self) -> bool
        {
            match self
                {
                    DebugMode::On =>
                        {
                            println!("Debug: ON");
                            let debug = true;
                            debug
                        }
                    DebugMode::Off =>
                        {
                            println!("Debug: OFF");
                            let debug = false;
                            debug
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
        fn server(self, file_info:&mut FileInfo, debug_mode:bool)
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
                                        println!("Connected");
                                        send_or_receive(file_info, &mut stream, &debug_mode);
                                    }
                                Err(e) =>
                                    {
                                        println!("Error: Can't Visit Stream -> {}", e);
                                        return;
                                    }
                            }
                    }
            }
        fn client(self, file_info:&mut FileInfo, debug_mode:bool)
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
                                send_or_receive(file_info, &mut stream, &debug_mode);
                            }
                        Err(e) =>
                            {
                                println!("Error: Connection -> {}", e);
                            }
                    }
                
            }
    }
fn send_or_receive(file_info:&mut FileInfo, stream:&mut TcpStream, debug_mode:&bool)
    {
        match &take_string("Input: Send 's', Receive 'r'".to_string())[..1]
        {
            "s" =>
                {
                    println!("Connected");
                    let start_time = Instant::now();
                    FileInfo::reading_operations(file_info, stream, &debug_mode);
                    let finish_time = Instant::now();
                    println!("Passed: Total -> {:#?}", finish_time.duration_since(start_time));
                }
            "r" =>
                {
                    let start_time = Instant::now();
                    FileInfo::writing_operations(file_info, stream, &debug_mode);
                    let finish_time = Instant::now();
                    println!("Passed: Total -> {:#?}", finish_time.duration_since(start_time));
                }
            input =>
                {
                    println!("Error: Give Valid Input, You Gave : {}", input);
                    panic!()                                                    }
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
fn debug_mod() -> DebugMode
    {
        match &take_string("Input: Debug -> On '1', Debug -> Off '0'".to_string())[0..1]
            {
                "1" => 
                    {
                        DebugMode::On
                    }
                "0" =>
                    {
                        DebugMode::Off
                    }
                input =>
                    {
                        println!("Error: Give Valid Input, You Gave : {}", input);
                        panic!()
                    }
            }
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
                          &mut data, DebugMode::debug_mode(debug_mod()));
                    },
                "c" => 
                    {
                        Connection::client
                        (Connection::Client(take_string("Input: Server IP Address to Connect".to_string()),
                         take_string("Input: Server Port Address to Connect".to_string())),
                          &mut data, DebugMode::debug_mode(debug_mod()));
                    }
                input => 
                    {
                        println!("Error: Give Valid Input, You Gave : {}", input);
                        return;
                    }
            }
    }
