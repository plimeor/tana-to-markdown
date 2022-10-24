use std::fs;
use std::path::Path;

pub struct Config {
    pub input: String,
    pub output: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("Missing arguments");
        }

        let input = args[1].clone();
        Config::check_input(&input)?;

        let output = args[2].clone();
        Config::check_output(&output)?;

        Ok(Config { input, output })
    }

    fn check_input(input: &String) -> Result<bool, &'static str> {
        let path = Path::new(input);
        let metadata = fs::metadata(path).expect("File not found");

        if !metadata.is_file() || !input.ends_with(".json") {
            return Err("Input file must be '.json'");
        }

        fs::OpenOptions::new()
            .read(true)
            .open(&input)
            .expect("Permission denied");

        Ok(true)
    }

    fn check_output(output: &String) -> Result<bool, &'static str> {
        let path = Path::new(&output);
        if path.extension().is_some() || path.exists() && fs::read_dir(path).unwrap().count() > 0 {
            return Err("output must be a emput dir");
        }

        Ok(true)
    }
}
