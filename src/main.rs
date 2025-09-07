use std::env::{self};
use std::fs::{self, File, Metadata};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Instant;

const BUFFER_SIZE: u64 = 100000;

#[derive(Debug)]
struct UserEnvironment {
    ip: IpAddr,
    port: u16,
    server: bool,
    send: bool,
    location: Option<String>,
    debug: bool,
}
impl UserEnvironment {
    fn new() -> UserEnvironment {
        UserEnvironment {
            ip: "127.0.0.1".parse().unwrap(),
            port: 2121,
            server: false,
            send: false,
            location: None,
            debug: false,
        }
    }
}
#[derive(Debug)]
struct FileInfo {
    file: Option<File>,
    location: Option<String>,
    sign: Option<String>,
    size_total: u64,
    size_current: u64,
    metadata: Option<Metadata>,
    progress: u8,
}
impl FileInfo {
    fn new() -> FileInfo {
        FileInfo {
            file: None,
            location: None,
            sign: None,
            size_total: 0,
            size_current: 0,
            metadata: None,
            progress: 0,
        }
    }
    fn pass_user_environment(&mut self, user_environment: &UserEnvironment) {
        self.location = user_environment.location.clone();
        self.sign = user_environment.location.clone();
    }
    fn reading_operations(&mut self, stream: &mut TcpStream, debug_mode: &bool) {
        match self.location.as_ref() {
            Some(_) => {
                self.read_metadata(debug_mode);
                match self.metadata {
                    Some(ref mut metadata) => {
                        if Metadata::is_symlink(metadata) {
                            //Unix-Windows Problem
                            println!("\n\tError: Symlink Transfers've not Supported yet\n");
                            //self.open_file(debug_mode);
                            //self.send_file(stream, &(100 as u8),debug_mode);
                        } else if Metadata::is_file(metadata) {
                            self.open_file(debug_mode);
                            self.send_file(stream, &(101_u8), debug_mode);
                        } else if Metadata::is_dir(metadata) {
                            //path recognition and creation on the other side
                            //std:path
                            println!("\n\tError: Folder Transfers've not Supported yet\n");
                            return;
                            //self.open_file(debug_mode);
                            //self.send_file(stream, &(102 as u8),debug_mode);
                        } else {
                            println!(
                                "Error: Undefined Type -> {}",
                                self.location.as_ref().unwrap()
                            );
                            return;
                        }
                    }
                    None => {
                        println!(
                            "Error: Read Metadata -> {}",
                            self.location.as_ref().unwrap()
                        );
                    }
                }
            }
            None => {
                println!("Error: Reading Operations -> {:#?}", &self.location);
                panic!();
            }
        }
    }
    fn writing_operations(&mut self, stream: &mut TcpStream, debug_mode: &bool) {
        self.write_file(stream, debug_mode);
        self.cleaning();
    }
    fn cleaning(&mut self) {
        self.location = self.sign.clone();
        self.size_current = 0;
    }
    fn read_metadata(&mut self, debug_mode: &bool) {
        let path = PathBuf::from(self.location.as_ref().unwrap());
        if path.is_symlink() {
            match path.symlink_metadata() {
                Ok(metadata) => {
                    self.metadata = Some(metadata);
                }
                Err(err_val) => {
                    println!(
                        "Error: Symlink Metadata -> {:#?} | Error: {}",
                        &self.location, err_val
                    );
                }
            }
        } else {
            match fs::metadata(self.location.as_ref().unwrap()) {
                Ok(metadata) => {
                    self.metadata = Some(metadata);
                    if *debug_mode {
                        println!("Done: Read Metadata -> {:#?}", self.metadata);
                    }
                }
                Err(err_val) => {
                    println!(
                        "Error: Read Metadata -> {} | Error: {}",
                        &self.location.as_ref().unwrap(),
                        err_val
                    );
                }
            }
        }
    }
    fn open_file(&mut self, debug_mode: &bool) {
        match File::options()
            .read(true)
            .write(true)
            .open(self.location.as_ref().unwrap())
        {
            Ok(file) => {
                self.file = Some(file);
                if *debug_mode {
                    println!("Done : Open File -> {:#?}", self.file);
                }
            }
            Err(err_val) => {
                println!(
                    "Error: Open File -> {} | Error: {}",
                    self.location.as_ref().unwrap(),
                    err_val
                );
            }
        }
    }
    fn send_file(&mut self, stream: &mut TcpStream, what_type: &u8, debug_mode: &bool) {
        self.size_total = self.metadata.as_ref().unwrap().len();
        let mut iteration = (self.size_total / BUFFER_SIZE) + 1;
        let total_iteration = iteration;
        let path_buf = PathBuf::from(Path::new(self.location.as_ref().unwrap()));
        match what_type {
            100 => {
                if *debug_mode {
                    println!(
                        "Done: Symlink Detected -> {}",
                        self.location.as_ref().unwrap()
                    );
                }
                self.callback_validation(stream, &String::from("100"), debug_mode);
            }
            101 => {
                if *debug_mode {
                    println!("Done: File Detected -> {}", self.location.as_ref().unwrap());
                }
                self.callback_validation(stream, &String::from("101"), debug_mode)
            }
            102 => {
                if *debug_mode {
                    println!(
                        "Done: Folder Detected -> {}",
                        self.location.as_ref().unwrap()
                    );
                }
                self.callback_validation(stream, &String::from("102"), debug_mode)
            }
            _ => {
                println!(
                    "Error: Undefined Type Detected ->{}",
                    self.location.as_ref().unwrap()
                );
                return;
            }
        }
        self.callback_validation(stream, &(self.size_total.to_string()), debug_mode);
        self.callback_validation(
            stream,
            &path_buf.file_name().unwrap().to_str().unwrap().to_string(),
            debug_mode,
        );

        self.show_info(&iteration, debug_mode);
        while iteration != 0 {
            iteration -= 1;
            let mut buffer = [0u8; BUFFER_SIZE as usize];
            if iteration != 0 {
                self.read_exact(&mut buffer, debug_mode);
            } else {
                self.read_exact(
                    &mut buffer[..(self.size_total % BUFFER_SIZE) as usize],
                    debug_mode,
                );
            }
            if *debug_mode {
                println!("Read Data = {:#?}", buffer);
            }
            self.send_exact(&buffer, stream, debug_mode);
            self.show_progress(iteration, total_iteration);
        }
    }
    fn callback_validation(&mut self, stream: &mut TcpStream, data: &String, debug_mode: &bool) {
        self.send_exact(format!("{}{}", data, "\n").as_bytes(), stream, debug_mode);
        match self.recv_until(stream, '\n', debug_mode) {
            Some(callback_callback) => {
                if callback_callback == data.to_string().as_bytes().to_vec() {
                    if *debug_mode {
                        println!("Done: Callback -> {}", self.location.as_ref().unwrap());
                        println!("{:#?} ", callback_callback);
                    }
                } else {
                    println!("Error: Callback -> {}", self.location.as_ref().unwrap());
                    println!("{:#?} ", callback_callback);
                    panic!()
                }
            }
            None => {
                panic!()
            }
        }
    }
    fn read_exact(&mut self, buffer: &mut [u8], debug_mode: &bool) {
        match self.file.as_ref().unwrap().read_exact(buffer) {
            Ok(_) => {
                if *debug_mode {
                    println!("Done: Read Bytes -> {}", self.location.as_ref().unwrap());
                    println!("{:#?}", buffer);
                }
            }
            Err(err_val) => {
                println!(
                    "Error: Read Bytes -> {} | Error: {}",
                    self.location.as_ref().unwrap(),
                    err_val
                );
                panic!()
            }
        }
    }
    fn send_exact(&mut self, buffer: &[u8], stream: &mut TcpStream, debug_mode: &bool) {
        let mut stream_writer = BufWriter::new(stream.try_clone().unwrap());
        match stream_writer.write_all(buffer) {
            Ok(_) => {
                self.size_current += buffer.len() as u64;
                if *debug_mode {
                    println!("Done: Send Bytes -> {:#?}", self.location);
                    println!("{:#?}", buffer);
                }
            }
            Err(err_val) => {
                println!(
                    "Error: Send Bytes -> {:#?} | Error: {}",
                    self.location, err_val
                );
                panic!();
            }
        }
        match stream_writer.flush() {
            Ok(_) => {
                if *debug_mode {
                    println!("Done: Flush -> {:#?}", self.location);
                }
            }
            Err(err_val) => {
                println!("Error: Flush -> {:#?} | Error: {}", self.location, err_val);
                panic!()
            }
        }
    }
    fn recv_exact(&mut self, buffer: &mut [u8], stream: &mut TcpStream, debug_mode: &bool) {
        match stream.read_exact(buffer) {
            Ok(_) => {
                self.size_current += buffer.len() as u64;
                if *debug_mode {
                    println!("Done: Receive Bytes -> {:#?}", self.location);
                    println!("{:#?}", buffer);
                }
            }
            Err(err_val) => {
                println!(
                    "Error: Receive Bytes -> {:#?} | Error: {}",
                    self.location, err_val
                );
                panic!();
            }
        }
    }
    fn recv_until(
        &mut self,
        stream: &mut TcpStream,
        until: char,
        debug_mode: &bool,
    ) -> Option<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut stream_reader = BufReader::new(stream.try_clone().unwrap());
        match stream_reader.read_until(until as u8, &mut buffer) {
            Ok(_) => {
                if *debug_mode {
                    println!("Done: Receive Until -> {:#?}", self.location);
                    println!("{:#?}", buffer);
                }
                buffer.pop();
            }
            Err(err_val) => {
                println!(
                    "Error: Receive Until -> {:#?} | Error: {}",
                    self.location, err_val
                );
                return None;
            }
        }
        Some(buffer)
    }
    fn forge_file(&mut self, location: String, debug_mode: &bool) {
        //dont forget
        //directory recognition required for received location
        match self.location.as_ref() {
            Some(self_location) => {
                let mut path = PathBuf::from(&self_location);
                path.push(location);
                self.forge_folder(self_location.clone(), debug_mode);
                self.location = Some(path.to_str().unwrap().to_string());
            }
            None => {
                self.location = Some(location);
            }
        }
        match File::create(self.location.as_ref().unwrap()) {
            Ok(file) => {
                if *debug_mode {
                    println!("Done Forge File -> {:#?}", file);
                }
            }
            Err(err_val) => {
                println!(
                    "Error: Forge File -> {:#?} | Error: {}",
                    self.location.as_ref(),
                    err_val
                );
            }
        }
    }
    fn forge_folder(&mut self, location: String, debug_mode: &bool) {
        match fs::create_dir_all(&location) {
            Ok(_) => {
                if *debug_mode {
                    println!("Done: Forge Folder -> {}", &location);
                }
            }
            Err(err_val) => {
                println!("Error: Forge Folder -> {} | Error: {}", location, err_val);
            }
        }
    }
    fn callback_recv(&mut self, stream: &mut TcpStream, debug_mode: &bool) -> String {
        match self.recv_until(stream, '\n', debug_mode) {
            Some(mut callback) => {
                if *debug_mode {
                    println!("Done: Callback -> {:#?}", self.location);
                    println!("{:#?} ", callback);
                }
                let data = String::from_utf8(callback.clone()).unwrap();
                callback.push(b'\n');
                self.send_exact(callback.as_slice(), stream, debug_mode);
                data
            }
            None => {
                println!("Error: Callback -> {:#?}", self.location);
                panic!();
            }
        }
    }
    fn save_exact(&mut self, buffer: &[u8], debug_mode: &bool) {
        let mut file_writer = BufWriter::new(self.file.as_ref().unwrap());
        if *debug_mode {
            println!("{:#?}", file_writer);
        }
        match file_writer.write_all(buffer) {
            Ok(_) => {
                if *debug_mode {
                    println!(
                        "Done: Write -> {} | {} bytes",
                        self.location.as_ref().unwrap(),
                        self.size_current
                    );
                    println!("{:#?}", buffer);
                }
            }
            Err(err_val) => {
                println!(
                    "Error: Write -> {} | Error: {}",
                    self.location.as_ref().unwrap(),
                    err_val
                );
                panic!();
            }
        }
        match file_writer.flush() {
            Ok(_) => {
                if *debug_mode {
                    println!("Done: Flush -> {}", self.location.as_ref().unwrap());
                }
            }
            Err(err_val) => {
                println!(
                    "Error: Flush -> {} | Error: {}",
                    self.location.as_ref().unwrap(),
                    err_val
                );
                panic!();
            }
        }
    }
    fn write_file(&mut self, stream: &mut TcpStream, debug_mode: &bool) {
        let what_type: u8 = self.callback_recv(stream, debug_mode).parse().unwrap();
        self.size_total = self.callback_recv(stream, debug_mode).parse().unwrap();
        let location: String = self.callback_recv(stream, debug_mode);
        match what_type {
            100 => {
                if *debug_mode {
                    println!(
                        "Done: Symlink Detected -> {}",
                        self.location.as_ref().unwrap()
                    );
                }
                self.forge_file(location, debug_mode);
                return;
            }
            101 => {
                if *debug_mode {
                    println!("Done: File Detected -> {}", self.location.as_ref().unwrap());
                }
                self.forge_file(location, debug_mode);
            }
            102 => {
                if *debug_mode {
                    println!(
                        "Done: Folder Detected -> {}",
                        self.location.as_ref().unwrap()
                    );
                }
                self.forge_file(location, debug_mode);
            }
            _ => {
                println!(
                    "Error: Undefined Type -> {}",
                    self.location.as_ref().unwrap()
                );
                return;
            }
        }
        self.open_file(debug_mode);
        let mut iteration: u64 = (self.size_total / BUFFER_SIZE) + 1;
        let total_iteration = iteration;
        self.show_info(&iteration, debug_mode);
        while iteration != 0 {
            iteration -= 1;
            let mut buffer = [0u8; BUFFER_SIZE as usize];
            self.recv_exact(&mut buffer, stream, debug_mode);
            if iteration != 0 {
                self.save_exact(&buffer, debug_mode);
            } else {
                self.save_exact(
                    &buffer[..(self.size_total % BUFFER_SIZE) as usize],
                    debug_mode,
                );
            }
            self.show_progress(iteration, total_iteration);
        }
    }
    fn show_info(&mut self, iteration: &u64, debug_mode: &bool) {
        println!("File = {}", self.location.as_ref().unwrap());
        println!("Size = {}", self.size_total);
        if *debug_mode {
            println!("Iteration = {}", iteration);
        }
    }
    fn show_progress(&mut self, iteration: u64, total_iteration: u64) {
        if iteration % 10 == 0 {
            let progress: u8 =
                100_u8 - ((iteration as f64 / total_iteration as f64) * 100_f64) as u8;
            if progress != self.progress {
                self.progress = progress;
                println!("%{}", self.progress);
            }
        }
    }
}
enum Connection {
    Server(String, String),
    Client(String, String),
}

impl Connection {
    fn server(self, file_info: &mut FileInfo, user_environment: &UserEnvironment) {
        print!("Server -> ");
        if user_environment.debug {
            println!("{:#?}", user_environment);
            println!("{:#?}", file_info);
        }
        let ip: String;
        let port: String;
        let address: String;
        match self {
            Connection::Server(in1, in2) => {
                ip = in1.trim_end().to_string();
                port = in2.trim_end().to_string();
                address = format!("{}:{}", ip, port);
                println!("{}", address);
            }
            _ => return,
        }
        let socket = TcpListener::bind(address);
        for stream in socket.expect("Error: Can't Check Connections").incoming() {
            match stream {
                Ok(mut stream) => {
                    println!("Connected");
                    send_or_receive(
                        file_info,
                        &mut stream,
                        &user_environment.debug,
                        user_environment,
                    );
                }
                Err(e) => {
                    println!("Error: Can't Visit Stream -> {}", e);
                    return;
                }
            }
        }
    }
    fn client(self, file_info: &mut FileInfo, user_environment: &UserEnvironment) {
        print!("Client -> ");
        if user_environment.debug {
            println!("{:#?}", user_environment);
            println!("{:#?}", file_info);
        }
        let ip: String;
        let port: String;
        let address: String;
        match self {
            Connection::Client(in1, in2) => {
                ip = in1.trim_end().to_string();
                port = in2.trim_end().to_string();
                address = format!("{}:{}", ip, port);
                println!("{}", address);
            }
            _ => return,
        }
        match TcpStream::connect(address) {
            Ok(mut stream) => {
                println!("Connected");
                send_or_receive(
                    file_info,
                    &mut stream,
                    &user_environment.debug,
                    user_environment,
                );
            }
            Err(e) => {
                println!("Error: Connection -> {}", e);
            }
        }
    }
}
fn send_or_receive(
    file_info: &mut FileInfo,
    stream: &mut TcpStream,
    debug_mode: &bool,
    user_environment: &UserEnvironment,
) {
    match user_environment.send {
        true => {
            let start_time = Instant::now();
            FileInfo::reading_operations(file_info, stream, debug_mode);
            let finish_time = Instant::now();
            println!("Done: Transfer");
            println!(
                "Passed: Total -> {:#?}",
                finish_time.duration_since(start_time)
            );
        }
        false => {
            let start_time = Instant::now();
            FileInfo::writing_operations(file_info, stream, debug_mode);
            let finish_time = Instant::now();
            println!("Done: Transfer");
            println!(
                "Passed: Total -> {:#?}",
                finish_time.duration_since(start_time)
            );
        }
    }
}
fn take_args(mut user_environment: UserEnvironment) -> Option<UserEnvironment> {
    let env_args: Vec<String> = env::args().collect();
    if env_args.len() > 16 {
        println!(
            "Error: Too Many Arguments, You Gave {} Arguments",
            env_args.len()
        );
        return None;
    }
    let mut i = 1;
    while i < env_args.len() {
        match env_args[i].as_str() {
            "--ip" | "-i" => {
                user_environment.ip = env_args[i + 1].parse().unwrap();
                i += 1;
            }
            "--port" | "-p" => {
                user_environment.port = env_args[i + 1].parse().unwrap();
                i += 1;
            }
            "--location" | "-l" => {
                user_environment.location = Some(env_args[i + 1].parse().unwrap());
                i += 1;
            }
            "--server" | "-sv" => {
                user_environment.server = true;
            }
            "--client" | "-cl" => {
                user_environment.server = false;
            }
            "--send" | "-s" => {
                user_environment.send = true;
            }
            "--receive" | "-r" => {
                user_environment.send = false;
            }
            "--debug" | "-d" => {
                user_environment.debug = true;
            }
            "--help" | "-h" => {
                show_help();
                return None;
            }
            err => {
                println!("Error: Invalid Argument, You Gave {}", err);
                return None;
            }
        }
        i += 1;
    }
    Some(user_environment)
}
fn show_help() {
    println!("\n\n\n");
    println!("   Arguments          |  Details                    |  Defaults");
    println!("----------------------------------------------------------------------");
    println!("   -i  -> --ip        |  Specifies IP Address       |  127.0.0.1");
    println!("   -p  -> --port      |  Specifies Port Address     |  2121");
    println!("   -l  -> --location  |  Specifies Location Address |  Same as Program");
    println!("   -sv -> --server    |  Starts as a Server         |  False");
    println!("   -cl -> --client    |  Starts as a Client         |  True");
    println!("   -s  -> --send      |  Starts as a Sender         |  False");
    println!("   -r  -> --receive   |  Starts as a Receiver       |  True");
    println!("   -d  -> --debug     |  Starts in Debug Mode       |  False");
    println!("   -h  -> --help      |  Shows Help                 |  False");
    println!("\n\n\n");
}
fn main() {
    //DONT FORGET
    //First we should check folder structure and validation then make connection.
    //Until's can be deprecated, 100k byte should be enough for eveything.(Security)
    println!("Hello, world!");
    let mut file_info: FileInfo = FileInfo::new();
    let user_environment: UserEnvironment = match take_args(UserEnvironment::new()) {
        Some(usr_env) => usr_env,
        None => {
            return;
        }
    };
    file_info.pass_user_environment(&user_environment);
    match user_environment.server {
        true => {
            Connection::server(
                Connection::Server(
                    user_environment.ip.to_string(),
                    user_environment.port.to_string(),
                ),
                &mut file_info,
                &user_environment,
            );
        }
        false => {
            Connection::client(
                Connection::Client(
                    user_environment.ip.to_string(),
                    user_environment.port.to_string(),
                ),
                &mut file_info,
                &user_environment,
            );
        }
    }
}
