use std::fs::{File, Metadata, self};
use std::time::Instant;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, self, BufReader};
use std::env::{self};

struct FileInfo
    {
        file:Option<File>,
        reader:Option<BufReader<File>>,
        location:String,
        bytes_current:Vec<u8>,
        size_current:usize,
        size_total:Option<usize>,
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
                                        self.size_total = Some(Metadata::len(&metadata) as usize);
                                        self.read_file();
                                    }
                                else if Metadata::is_symlink(metadata)
                                    {
                                        self.size_total = Some(Metadata::len(&metadata) as usize);
                                        self.read_file();
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
                match self.size_total
                    {
                        Some(ref mut size_total) =>
                            {

                                self.file_to_byte(stream)
                            }
                        None =>
                            {
                                println!("Error: Read Size -> {}", self.location);
                            }
                    }
            }
        fn writing_operations(&mut self, stream:&mut TcpStream)
            {
                self.write_file();
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
                                self.reader = Some(BufReader::new(file));
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Open File -> {} | Error: {}", self.location, err_val);
                            }
                    }
                
            }
        fn file_to_byte(&mut self, stream:&mut TcpStream)
            {
                self.bytes_current.clear();
                match self.file 
                    {
                        Some(ref mut file_descriptor) => 
                            {
                                //replace with readbuffer ?
                                match self.reader
                                    {
                                        Some(reader) =>
                                            {
                                                // I don't know what to do
                                            }
                                        None =>
                                            {

                                            }
                                    }
                                match File::read_exact(file_descriptor, &mut self.bytes_current)
                                    {
                                        Ok(_) => 
                                            {
                                                self.size_current = self.size_current+self.bytes_current.len();
                                                //need to send data, or call here from connection
                                                match stream.write_all(&self.bytes_current)
                                                    {
                                                        Ok(_) =>
                                                            {
                                                                // We will track when we stop later
                                                            }
                                                        Err(err_val) =>
                                                            {
                                                                println!("Error: Send -> {} | Error: {}", self.location, err_val)
                                                            }
                                                    }
                                                if self.size_total == Some(self.size_current)
                                                    {
                                                        println!("Done: Read -> {} | Done: Read -> {} bytes", self.location, self.size_current);
                                                    }
                                                else 
                                                    {
                                                        self.file_to_byte(stream);
                                                    }
                                            }
                                        Err(err_val) => println!("Error: Read -> {} | Error: {}", self.location, err_val),
                                    }
                            }
                        None =>
                            {
                                println!("Error: File None -> {}", self.location);
                            }
                    }
            }
        fn write_file(&mut self)
            {
                //use match, there is a chance to fail creation. don't pass with just some
                self.file = Some(File::create(&self.location).expect("Error: Create File"));
                match self.file
                    {
                        Some(ref mut file_descriptor) =>
                            {
                                match File::write_all(file_descriptor, &mut self.bytes)
                                    {
                                        Ok(_) =>
                                            {
                                                self.size_total = self.bytes.len();
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
                                                                FileInfo::writing_operations(file_info, &mut stream);
                                                                let finish_time = Instant::now();
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
                reader:None,
                location:take_arg(),
                bytes_current:Vec::with_capacity(1024),
                size_current:0 as usize,
                size_total:None,
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
