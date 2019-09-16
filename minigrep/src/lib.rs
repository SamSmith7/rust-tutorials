use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;


pub struct Config {
    pub case_sensitive: bool,
    pub query: String,
    pub filename: String
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {

        args.next();

        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a query string"),
        };

        let filename = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a query string"),
        };

        let case_sensitive = env::var("CASE_INSENSITIVE").is_err();

        Ok(Config {case_sensitive, query, filename})
    }
}

pub fn run(config: Config) -> Result<(), Box<Error>> {

    let mut f = File::open(config.filename)?;
    let mut contents = String::new();

    f.read_to_string(&mut contents)?;

    let results = if config.case_sensitive {
        search(&config.query, &contents)
    } else {
        case_insensitive_search(&config.query, &contents)
    };

    for line in results {
        println!("{}", line);
    }

    Ok(())
}

pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {

    contents.lines()
        .filter(|line| line.contains(query))
        .collect()
}

pub fn case_insensitive_search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {

    let query = query.to_lowercase();

    contents.lines()
        .filter(|line| line.to_lowercase().contains(&query))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn one_result() {

        let query = "duct";
        let contents = "/
Rust:
safe, fast, productive.
Pick three.
Duct tape.";

        assert_eq!(
            vec!["safe, fast, productive."],
            search(query, contents)
        );
    }

    #[test]
    fn case_insensitive() {

        env::set_var("CASE_INSENSITIVE", "true");
        let query = "rUsT";
        let contents = "/
Rust:
safe, fast, productive.
Pick three.
Trust Me";

        assert_eq!(
            vec!["Rust:", "Trust Me"],
            search(query, contents)
        );

        env::remove_var("CASE_INSENSITIVE");
    }
}
