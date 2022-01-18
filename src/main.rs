use std::collections::HashMap;
use std::{env, fs, io, io::prelude::*};
use walkdir::{DirEntry, WalkDir};
use colored::*;
use std::process::exit;
use regex::Regex;


struct Config {
    files_to_ignore: Vec<String>,
    files_to_search: Vec<String>,
    search_term: Regex,
    directory: String,
    has_directory: bool,
    has_string: bool,
    recursive: bool,
    debug: bool,
    check_extra: bool,
    out_to_file: bool,
    save_file_path: String,
}

impl Config {
    fn new() -> Config {
        Config {
            files_to_ignore: Vec::new(),
            files_to_search: Vec::new(),
            search_term: Regex::new("").unwrap(),
            directory: String::new(),
            has_directory: false,
            has_string: false,
            recursive: false,
            debug: false,
            check_extra: false,
            out_to_file: false,
            save_file_path: String::new(),
        }
    }
}


/// Check if file is hidden or not
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

// Check if file 
fn check_if_ignore(file_name: &str, ignore_types: Vec<String>) -> bool {
    for ignore_type in ignore_types {
        if !ignore_type.is_empty() && file_name.ends_with(ignore_type.as_str()) {
            return true;
        }
    }
    return false;
}

fn check_if_include(file_name: &str, files_to_search: &Vec<String>) -> bool {
    for file_type in files_to_search.iter() {
        if file_name.ends_with(file_type.as_str()) {
            return true;
        }
    }
    return false;
}

fn with_quotes(s: &String) -> String {
    let q = "\"";
    format!("{}{}{}", q, s, q)
}

fn msg(s: &str) {
    io::stdout().flush().unwrap();
    print!("\r{}{}", s,"            ",);
    io::stdout().flush().unwrap();
}

fn give_half_or(s: &String, len: usize) -> String{
    let mut result = s.clone();
    if result.len() > len {
        result = result.chars().take(len).collect::<String>();
        result.push_str("...");
        return result;
    }else{
        result
    }
    
}

fn get_file_name(s: &String) -> String {
    let res = s.replace("\\", "/");
    res.split("/").last().unwrap().to_string()
}

fn search_in_file(conf: &Config) -> HashMap<String, Vec<String>> {
    let mut file_data: HashMap<String, Vec<String>> = HashMap::new();

    const TOTAL_LEN:usize = 90;

    if conf.has_directory == true {
        
        println!(
            "{} {} {} {}, {}={}",
            "Searching in:".cyan().bold(),
            with_quotes(&conf.directory).green().bold(),
            "for:".cyan(),
            with_quotes(&conf.search_term.to_string()).green().bold(),
            "recursive".cyan().bold().underline(),
            &conf.recursive.to_string().red()
        );
        if conf.recursive {
            let walker = WalkDir::new(conf.directory.clone()).into_iter();
            for entry in walker
                .filter_entry(|e| {
                    !is_hidden(e)
                        && !check_if_ignore(
                            e.file_name().to_str().unwrap(),
                            conf.files_to_ignore.clone(),
                        )
                })
                .filter_map(|e| e.ok())
            {
                if entry.metadata().unwrap().is_file() {
                    let file_path = entry.path().display().to_string();
                    let name = entry.path().file_name().unwrap().to_str().unwrap().to_string();
                    if conf.check_extra {
                        if !check_if_include(&file_path, &conf.files_to_search) {
                            continue;
                        }
                    }
                    let content = fs::read_to_string(&file_path);

                    match content {
                        Ok(content) => {
                            let lines = content.lines();
                            for (i, line) in lines.enumerate() {
                                msg(format!(
                                    "{} {}",
                                    "Finding in:".cyan(),
                                    with_quotes(&name).green().bold()
                                ).as_str());

                                if conf.search_term.is_match(line) {
                                   let line_to_show = give_half_or(&line.to_string(), TOTAL_LEN);
                            
                                    file_data.insert(
                                        file_path.clone(),
                                        vec![(i as i32 + 1).to_string(), line_to_show],
                                    );

                                    msg(format!(
                                        "{} {}",
                                        "Found:".cyan(),
                                        with_quotes(&name).green().bold()
                                    ).as_str());

                                }
                            }
                        }
                        Err(e) => {
                            if conf.debug {
                                println!("Error reading file '{}': {}", &file_path, e);
                                
                            }
                            continue;
                        }
                    }
                }
       
            }
        } else {
            let paths = fs::read_dir(&conf.directory).unwrap();
            for file in paths {
                let file_name = &file.unwrap().path().display().to_string();
                let md = fs::metadata(&file_name);
                match md {
                    Ok(md) => {
                        if md.is_file() {

                            if check_if_ignore(file_name, conf.files_to_ignore.clone()) {
                                continue;
                            }

                            if conf.check_extra {
                                if !check_if_include(file_name, &conf.files_to_search) {
                                    continue;
                                }
                            }

                            let content = fs::read_to_string(&file_name);

                            match content {
                                Ok(content) => {
                                    let lines = content.lines();
                                    msg(format!(
                                        "{} {}",
                                        "Finding in:".cyan(),
                                        with_quotes(&give_half_or(&get_file_name(&file_name), 30)).green().bold()
                                    ).as_str());
    
                                    for (i, line) in lines.enumerate() {
                                        if conf.search_term.is_match(line) {

                                            let line_to_show = give_half_or(&line.to_string(), TOTAL_LEN);

                                            file_data.insert(
                                                file_name.clone(),
                                                vec![(i as i32 + 1).to_string(), line_to_show],
                                            );
                                            msg(format!(
                                                "{} {}",
                                                "Found:".cyan(),
                                                with_quotes(&file_name).green().bold()
                                            ).as_str());
                                        }
                                    }
                                }
                                Err(e) => {
                                    if conf.debug {
                                        println!("Error reading file '{}': {}", &file_name, e);
                                    }
                                    continue;
                                }
                            }
                        }
                    }

                    Err(e) => {
                        if conf.debug {
                            println!("Error reading file '{}': {}", &file_name, e);
                            continue;
                        }
                    }
                }
            }
        }
    }

    file_data
}

fn main() {

    let mut ignore_types: Vec<String> = vec![
        "zip".to_string(),
        "lib".to_string(),
        "so".to_string(),
        "dll".to_string(),
        "dylib".to_string(),
        "exe".to_string(),
        "pyc".to_string(),
        "pyo".to_string(),
        "bin".to_string(),
        "pdb".to_string(),
        "jpg".to_string(),
        "jpeg".to_string(),
        "png".to_string(),
        "gif".to_string(),
        "bmp".to_string(),
        "ico".to_string(),
        "tiff".to_string(),
        "mp3".to_string(),
        "wav".to_string(),
        "ogg".to_string(),
        "flac".to_string(),
        "mp4".to_string(),
        "mkv".to_string(),
        "avi".to_string(),
        "mov".to_string(),
        "wmv".to_string(),
        "flv".to_string(),
        "7zip".to_string(),
        "rar".to_string(),
        "gz".to_string(),
        "tar".to_string(),
        "tgz".to_string(),
        "blend".to_string(),
        "blend1".to_string(),
        "img".to_string(),
    ];

    let mut files_to_search = Vec::<String>::new();

    let mut conf = Config::new();

    let args = env::args().skip(1).collect::<Vec<_>>();


    let mut has_error = false;

    let help_text: String = r#"
       Usage:
              *Backslashes needs to be escaped with another backslash if in quotes for paths only.

              -> Search with default settings (recursive off):
                     <dir> <pattern>

              -> Search for sepcified file extensions separated with ',':
                     <dir> <pattern> --only txt,rs,toml

              -> Ignore extention separated with ',':
                     <dir> <pattern> --ignore "txt, cpp, py"

              -> -r to enable recursive search.

              -> --o to save output to a file
                     <dir> <pattern> --ignore txt,cpp,py -o output.txt

              -> -d to enable print errors if failed to open any file."#
        .to_string();

    if args.len() == 0 {
        println!("{}", help_text);
    } else {
        for (i, cmd) in args.iter().enumerate() {
            if cmd == "-help" {
                println!("{}", help_text);
            } else if cmd == "--o" {
                conf.out_to_file = true;
                let fname = &args[i + 1];
                conf.save_file_path = fname.clone();
            } else if cmd == "-r" {
                conf.recursive = true;
            } else if cmd == "-d" {
                conf.debug = true;
            } else if cmd == "--ignore" {
                let ignore_t = args[i + 1].split(",");
                for ignore_type in ignore_t {
                    ignore_types.push(ignore_type.trim().to_string());
                }
            } else if cmd == "--only" {
                let add_t = args[i + 1].split(",");
                for add_type in add_t {
                    if add_type.trim().is_empty() {
                        continue;
                    }
                    files_to_search.push(add_type.trim().to_string());
                }
                conf.check_extra = true;
            }else if cmd.starts_with("-"){
                println!("{} '{}'", "Invalid argument: ".red().bold(), cmd);
                println!("{}", help_text);
                has_error = true;
            } 
            else {
                if conf.has_directory == false {
                    conf.directory = cmd.to_string();
                    conf.has_directory = true;
                } else if conf.has_string == false {
                    conf.search_term = Regex::new(&cmd).unwrap();
                    conf.has_string = true;
                }
            }
        }
    }

    if has_error { exit(1); }
    
    conf.files_to_ignore = ignore_types;
    conf.files_to_search = files_to_search;

    let mut formatted: String = String::new();
    let result = search_in_file(&conf);

    for (file_name, data) in result.into_iter() {
        if conf.out_to_file {
            formatted += format!(
                "\n{}{}
                \rAt line: {}
                \rContent: {}
                \r---------------------------------------",
                "File: ",
                with_quotes(&file_name),
                data[0],
                with_quotes(&data[1])
            )
            .as_str();
        } else {
            formatted += format!(
                "\n{}{}
                \rAt line: {}
                \rContent: {}
                \r---------------------------------------",
                "File: ".red(),
                with_quotes(&file_name).green().bold(),
                data[0].red(),
                with_quotes(&data[1]).green().bold()
            )
            .as_str();
        }
    }

    if conf.out_to_file {
        // check if file exits or not if not create and write else append
        let save_file = env::current_dir()
            .unwrap()
            .as_path()
            .join(&conf.save_file_path);
        let mut file = match fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&save_file)
        {
            Ok(file) => file,
            Err(e) => {
                println!("Error Saving in file '{}': {}", &conf.save_file_path, e);
                return;
            }
        };

        match file.write_all(formatted.as_bytes()) {
            Ok(_) => {
                println!("{} {}", " Saved to file:".green(), &conf.save_file_path);
            }
            Err(e) => {
                println!(
                    "{} '{}': {}",
                    "Error Saving in file:".red(),
                    &conf.save_file_path,
                    e
                );
            }
        }
    } else {
        println!("{}", &formatted);
    }

}
