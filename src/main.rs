use std::fs::{File, Metadata, self};
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::net::{TcpListener, TcpStream, IpAddr};
use std::io::{Read, Write, BufWriter, BufReader, BufRead};
use std::env::{self};


const BUFFER_SIZE:u64 = 100000;

#[derive(Debug)]
struct UserEnvironment
    {
        ip:IpAddr,
        port:u16,
        server:bool,
        send:bool,
        location:Option<String>,
		debug:DebugMode,
    }
#[derive(Debug)]
struct FileInfo
    {
        file:Option<File>,
        location:Option<String>,
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
                                println!("Error: Read Metadata -> {}", self.location.as_ref().unwrap());
                            }
                    }
            }
        fn writing_operations(&mut self, stream:&mut TcpStream, debug_mode:&bool)
            {
                self.write_file(stream, debug_mode);
            }
        fn read_metadata(&mut self)
            {
                self.metadata = Some(fs::metadata(&self.location.as_ref().unwrap()).expect("Error: Read Metadata"));
            }
        fn open_file(&mut self)
            {
                match File::open(&self.location.as_ref().unwrap())
                    {
                        Ok(file) =>
                            {
                                self.file = Some(file);
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Open File -> {} | Error: {}", self.location.as_ref().unwrap(), err_val);
                                return;
                            }
                    }
                
            }
        fn send_file(&mut self, stream:&mut TcpStream, debug_mode:&bool)
            {
                let size = self.metadata.as_ref().unwrap().len();
                let mut iteration = (size/BUFFER_SIZE)+1;
                let total_iteration = iteration;
				let path_buf = PathBuf::from(Path::new(self.location.as_ref().unwrap()));
                self.callback_validation(stream, &(size.to_string()), debug_mode);
				self.callback_validation(stream, &path_buf.file_name().unwrap().to_str().unwrap().to_string(), debug_mode);
				println!("File = {}", self.location.as_ref().unwrap());
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
        fn callback_validation(&mut self, stream:&mut TcpStream, data:&String, debug_mode:&bool)
            {
                self.send_exact(String::from(data.clone() + "\n").as_bytes(), stream, debug_mode);
                match self.recv_until(stream, '\n', debug_mode)
                    {
                        Some(callback_callback) =>
                            {
                                if callback_callback == data.to_string().as_bytes().to_vec()
                                    {
                                        println!("Done: Callback -> {}", self.location.as_ref().unwrap());
                                        if *debug_mode
                                            {
                                                println!("{:#?} ", callback_callback);
                                            }
                                    }
                                else 
                                    {
                                        println!("Error: Callback -> {}", self.location.as_ref().unwrap());
                                        println!("{:#?} ", callback_callback);
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
                                        println!("Done: Read Bytes -> {}", self.location.as_ref().unwrap());
                                        println!("{:#?}", buffer);
                                    }
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Read Bytes -> {} | Error: {}", self.location.as_ref().unwrap(), err_val);
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
                                        println!("Done: Send Bytes -> {:#?}", self.location);
                                        println!("{:#?}", buffer);
                                    }
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Send Bytes -> {:#?} | Error: {}", self.location, err_val);
                                panic!();
                            }
                    }
                match stream_writer.flush()
                    {
                        Ok(_) =>
                            {
                                if *debug_mode
                                    {
                                        println!("Done: Flush -> {:#?}", self.location);
                                    }
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Flush -> {:#?} | Error: {}", self.location, err_val);
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
                                        println!("Done: Receive Bytes -> {:#?}", self.location);
                                        println!("{:#?}", buffer);
                                    }
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Receive Bytes -> {:#?} | Error: {}", self.location, err_val);
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
                                        println!("Done: Receive Until -> {:#?}", self.location);
                                        println!("{:#?}", buffer);
                                    }
                                buffer.pop();
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Receive Until -> {:#?} | Error: {}", self.location, err_val);
                                return None;
                            }
                    }
                return Some(buffer);
            }
        fn forge_file(&mut self)
            {
                match &self.location
                    {
                        Some(location) =>
                            {
                                self.file = Some(File::create(&location).expect("Error: Create File"));
                            }
                        None =>
                            {
                                println!("Error: Forge File -> {:#?}", self.location);
                                panic!();
                            }
                    }
            }
        fn callback_recv(&mut self, stream:&mut TcpStream, debug_mode:&bool) -> String
            {
                match self.recv_until(stream, '\n', debug_mode)
                    {
                        Some(mut callback) =>
                            {
                                println!("Done: Callback -> {:#?}", self.location);
                                if *debug_mode
                                    {
                                        println!("{:#?} ", callback);
                                    }
                                let data = String::from_utf8(callback.clone()).unwrap();
                                callback.push(b'\n');
                                self.send_exact(&callback.as_slice(), stream, debug_mode);
                                data
                            }
                        None =>
                            {
                                println!("Error: Callback -> {:#?}", self.location);
                                panic!();
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
                                        println!("Done: Write -> {} | {} bytes", self.location.as_ref().unwrap(), self.size_current);
                                        println!("{:#?}", buffer);
                                    }
                            }
                        Err(err_val) => 
                            {
                                println!("Error: Write -> {} | Error: {}", self.location.as_ref().unwrap(),err_val);
                                panic!();
                            }
                    }
                match file_writer.flush()
                    {
                        Ok(_) =>
                            {
                                if *debug_mode
                                    {
                                        println!("Done: Flush -> {}", self.location.as_ref().unwrap());
                                    }
                            }
                        Err(err_val) => 
                            {
                                println!("Error: Flush -> {} | Error: {}", self.location.as_ref().unwrap(),err_val);
                                panic!();
                            }
                    }
            }
        fn write_file(&mut self, stream:&mut TcpStream, debug_mode:&bool)
            {
                let size:u64 = self.callback_recv(stream, debug_mode).parse().unwrap();
				self.location = Some(self.callback_recv(stream, debug_mode));
				self.forge_file();
                let mut iteration:u64 = (size/BUFFER_SIZE)+1;
                let total_iteration = iteration;
				println!("File = {}", self.location.as_ref().unwrap());
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
#[derive(Debug)]
enum DebugMode
    {
        On,
        Off
    }
impl DebugMode {
    fn debug_mode(&self) -> bool
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
        fn server(self, file_info:&mut FileInfo, debug_mode:bool, user_environment:&UserEnvironment)
            {
                print!("Server -> ");
				if debug_mode
					{
						println!("{:#?}", user_environment);
						println!("{:#?}", file_info);
					}
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
                                        send_or_receive(file_info, &mut stream, &debug_mode, user_environment);
                                    }
                                Err(e) =>
                                    {
                                        println!("Error: Can't Visit Stream -> {}", e);
                                        return;
                                    }
                            }
                    }
            }
        fn client(self, file_info:&mut FileInfo, debug_mode:bool, user_environment:&UserEnvironment)
            {
                print!("Client -> ");
				if debug_mode
					{
						println!("{:#?}", user_environment);
						println!("{:#?}", file_info);
					}
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
                                send_or_receive(file_info, &mut stream, &debug_mode, user_environment);
                            }
                        Err(e) =>
                            {
                                println!("Error: Connection -> {}", e);
                            }
                    }
                
            }
    }
fn send_or_receive(file_info:&mut FileInfo, stream:&mut TcpStream, debug_mode:&bool, user_environment:&UserEnvironment)
    {
        match user_environment.send
        {
            true =>
                {
                    let start_time = Instant::now();
                    FileInfo::reading_operations(file_info, stream, &debug_mode);
                    let finish_time = Instant::now();
                    println!("Passed: Total -> {:#?}", finish_time.duration_since(start_time));
                }
            false =>
                {
                    let start_time = Instant::now();
                    FileInfo::writing_operations(file_info, stream, &debug_mode);
                    let finish_time = Instant::now();
                    println!("Passed: Total -> {:#?}", finish_time.duration_since(start_time));
                }
        }
    }
fn take_args(user_environment:&mut UserEnvironment)
    {
        let env_args:Vec<String> = env::args().collect();
		if env_args.len() > 15
			{
				println!("Error: Too Many Arguments, You Gave {} Arguments", env_args.len());
				panic!();
			}
		let mut i = 1;
		while i < env_args.len()
			{
				match env_args[i].as_str()
					{
						"--ip" =>
							{
								user_environment.ip = env_args[i+1].parse().unwrap();
								i += 1;
							}
						"--port" =>
							{
								user_environment.port = env_args[i+1].parse().unwrap();
								i += 1;
							}
						"--location" =>
							{
								user_environment.location = Some(env_args[i+1].parse().unwrap());
								i += 1;
							}
						"--server" =>
							{
								user_environment.server = true;
							}
						"--client" =>
							{
								user_environment.server = false;
							}
						"--send" =>
							{
								user_environment.send = true;
							}
						"--receive" =>
							{
								user_environment.send = false;
							}
						"--debug" =>
							{
								user_environment.debug = DebugMode::On;
							}
						err =>
							{
								println!("Error: Invalid Argument, You Gave {}", err);
							}
					}
				i += 1;
			}
    }
fn main() 
    {
        //DONT FORGET
        //First we should check folder structure and validation then make connection.
        //Until's can be deprecated, 100k byte should be enough for eveything.(Security)
        println!("Hello, world!");
        let mut user_environment = UserEnvironment
            {
                ip:"127.0.0.1".parse().unwrap(),
                port:2121,
                server:false,
                send:false,
                location:None,
				debug:DebugMode::Off,
            };
        take_args(&mut user_environment);
        let mut file_info = FileInfo
            {
                file:None,
                location:user_environment.location.clone(),
                size_current:0 as usize,
                metadata:None,
            };
        match user_environment.server
            {
                true => 
                    {
                        Connection::server
                        (Connection::Server(user_environment.ip.to_string(), user_environment.port.to_string()),
                          &mut file_info, DebugMode::debug_mode(&user_environment.debug), &user_environment);
                    },
                false => 
                    {
                        Connection::client
                        (Connection::Client(user_environment.ip.to_string(), user_environment.port.to_string()),
                          &mut file_info, DebugMode::debug_mode(&user_environment.debug), &user_environment);
                    }
            }
    }
