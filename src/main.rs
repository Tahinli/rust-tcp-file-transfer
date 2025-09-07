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
		debug:bool,
    }
#[derive(Debug)]
struct FileInfo
    {
        file:Option<File>,
        location:Option<String>,
        sign:Option<String>,
        size_current:usize,
        metadata:Option<Metadata>,
        progress:u8,
    }
impl FileInfo 
    {
        fn reading_operations(&mut self, stream:&mut TcpStream, debug_mode:&bool)
            {
                //Pathbuf Symlink Metadata
                //Pathbuf is_Symlink 
                match &self.location
                    {
                        Some(_) =>
                            {
                                self.read_metadata(debug_mode);
                                match self.metadata
                                    {
                                        Some(ref mut metadata) =>
                                            {
                                                if Metadata::is_symlink(metadata)
                                                    {
                                                        //Recursivity Problem
                                                        println!("\n\tError: Symlink Transfers've not Supported yet\n");
                                                        return;
                                                    }
                                                else if Metadata::is_file(metadata)
                                                    {
                                                        self.open_file(debug_mode);
                                                        self.send_file(stream, debug_mode);
                                                    }
                                                
                                                else 
                                                    {
                                                        //path recognition and creation on the other side
                                                        //std:path
                                                        println!("\n\tError: Folder Transfers've not Supported yet\n");
                                                        return;
                                                    }
                                            }
                                        None =>
                                            {
                                                println!("Error: Read Metadata -> {:#?}", &self.location);
                                            }
                                    }
                            }
                        None =>
                            {
                                println!("Error: Reading Operations -> {:#?}", &self.location);
                                panic!();
                            }
                    }
            }
        fn writing_operations(&mut self, stream:&mut TcpStream, debug_mode:&bool)
            {
                self.write_file(stream, debug_mode);
                self.clean_sign();
            }
        fn clean_sign(&mut self)
            {
                self.location = self.sign.clone();
            }
        fn read_metadata(&mut self, debug_mode:&bool)
            {
                match fs::metadata(&self.location.as_ref().unwrap())
                    {
                        Ok(metadata) =>
                            {
                                self.metadata = Some(metadata);
                                if *debug_mode
                                    {
                                        println!("Done: Read Metadata -> {:#?}", self.metadata);
                                    }
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Read Metadata -> {} | Error: {}", &self.location.as_ref().unwrap(), err_val);
                            }
                    }
            }
        fn open_file(&mut self,debug_mode:&bool)
            {
                match File::options().read(true).write(true).open(self.location.as_ref().unwrap())
                    {
                        Ok(file) =>
                            {
                                self.file = Some(file);
                                if *debug_mode
                                    {
                                        println!("Done : Open File -> {:#?}", self.file);
                                    }
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
				self.show_info(size, &iteration, debug_mode);
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
                        self.show_progress(iteration, total_iteration);
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
                                        if *debug_mode
                                            {
                                                println!("Done: Callback -> {}", self.location.as_ref().unwrap());
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
        fn forge_file(&mut self, location:String, debug_mode:&bool)
            {
                //dont forget
                //directory recognition required for received location
                match self.location.as_ref()
                    {
                        Some(self_location) =>
                            {
                                let mut path = PathBuf::from(&self_location);
                                path.push(location);
                                self.forge_folder(self_location.clone(), debug_mode);
                                self.location = Some(path.to_str().unwrap().to_string());            
                            }
                        None =>
                            {
                                self.location = Some(location);
                            }
                    }
                match File::create(self.location.as_ref().unwrap())
                    {
                        Ok(file) =>
                            {
                                if *debug_mode
                                    {
                                        println!("Done Forge File -> {:#?}", file);
                                    }
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Forge File -> {:#?} | Error: {}", self.location.as_ref(), err_val);
                            }
                    }
            }
        fn forge_folder(&mut self, location:String, debug_mode:&bool)
            {
                match fs::create_dir_all(&location)
                    {
                        Ok(_) =>
                            {
                                if *debug_mode
                                    {
                                        println!("Done: Forge Folder -> {}", &location);
                                    }
                            }
                        Err(err_val) =>
                            {
                                println!("Error: Forge Folder -> {} | Error: {}", location, err_val);
                            }
                    }
            }
        fn callback_recv(&mut self, stream:&mut TcpStream, debug_mode:&bool) -> String
            {
                match self.recv_until(stream, '\n', debug_mode)
                    {
                        Some(mut callback) =>
                            {
                                
                                if *debug_mode
                                    {
                                        println!("Done: Callback -> {:#?}", self.location);
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
                if *debug_mode
                    {
                        println!("{:#?}", file_writer);
                    }
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
				let location:String = self.callback_recv(stream, debug_mode);
				self.forge_file(location, debug_mode);
                self.open_file(debug_mode);
                let mut iteration:u64 = (size/BUFFER_SIZE)+1;
                let total_iteration = iteration;
				self.show_info(size, &iteration, debug_mode);
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
                        self.show_progress(iteration, total_iteration);
                    }            
            }
        fn show_info(&mut self, size:u64, iteration:&u64, debug_mode:&bool)
            {
                println!("File = {}", self.location.as_ref().unwrap());
                println!("Size = {}", size);
                if *debug_mode
                    {
                        println!("Iteration = {}", iteration);
                    }
            }
        fn show_progress(&mut self, iteration:u64, total_iteration:u64)
            {
                if iteration%10 == 0
                    {
                        let progress:u8 = 100 as u8 - ((iteration as f64/total_iteration as f64)*100 as f64)as u8;
                        if progress != self.progress
                            {
                                self.progress = progress;
                                println!("%{}", self.progress);
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
        fn server(self, file_info:&mut FileInfo, user_environment:&UserEnvironment)
            {
                print!("Server -> ");
				if user_environment.debug
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
                                        send_or_receive(file_info, &mut stream, &user_environment.debug, user_environment);
                                    }
                                Err(e) =>
                                    {
                                        println!("Error: Can't Visit Stream -> {}", e);
                                        return;
                                    }
                            }
                    }
            }
        fn client(self, file_info:&mut FileInfo, user_environment:&UserEnvironment)
            {
                print!("Client -> ");
				if user_environment.debug
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
                                send_or_receive(file_info, &mut stream, &user_environment.debug, user_environment);
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
                    println!("Done: Transfer");
                    println!("Passed: Total -> {:#?}", finish_time.duration_since(start_time));
                }
            false =>
                {
                    let start_time = Instant::now();
                    FileInfo::writing_operations(file_info, stream, &debug_mode);
                    let finish_time = Instant::now();
                    println!("Done: Transfer");
                    println!("Passed: Total -> {:#?}", finish_time.duration_since(start_time));
                }
        }
    }
fn take_args(user_environment:&mut UserEnvironment) -> bool
    {
        let env_args:Vec<String> = env::args().collect();
		if env_args.len() > 16
			{
				println!("Error: Too Many Arguments, You Gave {} Arguments", env_args.len());
				return false;
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
								user_environment.debug = false;
							}
                        "--help" =>
                            {
                                show_help();
                                return false;

                            }
						err =>
							{
								println!("Error: Invalid Argument, You Gave {}", err);
                                return false;
							}
					}
				i += 1;
			}
        true
    }
fn show_help()
    {
        println!("\n\n\n");
        println!("  Arguments   |  Details                   |  Defaults");
        println!("--------------------------------------------------------------");
        println!("  --ip        |  Specifies IP Address       |  127.0.0.1");
        println!("  --port      |  Specifies Port Address     |  2121");
        println!("  --location  |  Specifies Location Address |  Same as Program");
        println!("  --server    |  Starts as a Server         |  False");
        println!("  --client    |  Starts as a Client         |  True");
        println!("  --send      |  Starts as a Sender         |  False");
        println!("  --receive   |  Starts as a Receiver       |  True");
        println!("  --debug     |  Starts in Debug Mode       |  False");
        println!("  --help      |  Shows Help                 |  False");
        println!("\n\n\n");
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
				debug:false,
            };
        if !take_args(&mut user_environment)
            {
                return;
            }
        let mut file_info = FileInfo
            {
                file:None,
                location:user_environment.location.clone(),
                sign:user_environment.location.clone(),
                size_current:0 as usize,
                metadata:None,
                progress:0,
            };
        match user_environment.server
            {
                true => 
                    {
                        Connection::server
                        (Connection::Server(user_environment.ip.to_string(), user_environment.port.to_string()),
                          &mut file_info, &user_environment);
                    },
                false => 
                    {
                        Connection::client
                        (Connection::Client(user_environment.ip.to_string(), user_environment.port.to_string()),
                          &mut file_info, &user_environment);
                    }
            }
    }
