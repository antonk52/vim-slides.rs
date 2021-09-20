use clap::{App, Arg};
use notify::{watcher, RecursiveMode, Watcher};
use pad::{Alignment, PadStr};
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::vec;

pub struct Slide {
    pub title: String,
    pub content: Vec<String>,
    pub comments: Vec<String>,
}

impl Slide {
    fn new() -> Slide {
        Slide {
            title: "".to_owned(),
            content: vec![],
            comments: vec![],
        }
    }
}

fn split_to_slides(contents: &str) -> Vec<Slide> {
    let lines: Vec<&str> = contents.split_terminator('\n').collect();
    let mut slides: Vec<Slide> = vec![];
    let mut current_slide = Slide::new();

    for line in lines {
        let trimmed = line.trim();

        if trimmed.starts_with('#') {
            if !current_slide.title.is_empty() || !current_slide.content.is_empty() {
                slides.push(current_slide);

                // reset "current_slide" value
                current_slide = Slide::new();
            }

            current_slide.title = trimmed.to_owned();
        } else if let Some(uncomment_line) = trimmed.strip_prefix("<!--") {
            // TODO support multiline comments

            match uncomment_line.strip_suffix("-->") {
                Some(comment_body) => current_slide.comments.push(comment_body.trim().to_owned()),
                None => current_slide
                    .comments
                    .push(uncomment_line.trim().to_owned()),
            };
        } else {
            current_slide.content.push(trimmed.to_owned());
        }
    }

    if !current_slide.title.is_empty() {
        slides.push(current_slide);
    }

    slides
}

pub struct VimSlidesArgs {
    pub source_file: String,
    pub destination: String,
}

fn create_slides_from_path(source: &str, dest: &str, verbose: bool) -> std::io::Result<()> {
    let contents =
        fs::read_to_string(source).expect("Something went wrong reading the source file");

    let slides = split_to_slides(contents.as_str());

    let dest_path = Path::new(dest);
    if dest_path.exists() {
        if !dest_path.is_dir() {
            eprintln!("Destination exists, but it is a file, not a directory");
            process::exit(1);
        } else {
            // TODO erase dir contents, prompt user?
            if verbose {
                println!("Destination dir already exists");
            }
        }
    } else {
        fs::create_dir(dest_path)?;
        println!("Created directory: {}", dest);
    }

    if verbose {
        println!("slides qty {}", slides.len());
    }

    for (i, slide) in slides.iter().enumerate() {
        let pad_char = "0".chars().next().unwrap();
        let slide_id = format!("{}", i + 1)
            .to_string()
            .pad(3, pad_char, Alignment::Right, true);
        let slide_filepath = dest_path.join(format!("{}.md", slide_id));

        let lines = slide.content.join("\n");
        let slide_content = format!("{}\n{}", slide.title, lines);

        fs::write(slide_filepath, slide_content.trim()).expect("Could not write file for a slide");
    }

    let editor: String = match env::var("EDITOR") {
        Ok(x) => x,
        Err(_) => String::from("vi"),
    };
    let slides_glob = dest_path
        .join("*")
        .into_os_string()
        .into_string()
        .expect("* is a string");

    println!("Done, to open slides run: {} {}", editor, slides_glob);
    if verbose {
        println!("navigate between slides with :next and :prev\n");
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let matches = App::new("vim-slides")
        .version("1.0.0")
        .author("Anton Kastritskiy")
        .about("Generates markdown slides for a vim presentation")
        .arg(
            Arg::with_name("source")
                .help("source markdown file")
                .required(true),
        )
        .arg(
            Arg::with_name("destination")
                .help("path to where the slides go")
                .takes_value(true)
                .required(false)
                .default_value("./slides"),
        )
        .arg(
            Arg::with_name("watch")
                .help("watch for file changes in the SOURCE file")
                .short("w")
                .long("watch"),
        )
        .get_matches();

    let dest_path_str = matches.value_of("destination").unwrap();

    let source_filepath = matches.value_of("source").unwrap();

    create_slides_from_path(source_filepath, dest_path_str, true)?;

    if matches.is_present("watch") {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
        watcher
            .watch(source_filepath, RecursiveMode::Recursive)
            .unwrap();

        println!("Waiting for changes for: {}", source_filepath);

        loop {
            match rx.recv() {
                Ok(_) => {
                    create_slides_from_path(source_filepath, dest_path_str, false)?;
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    }

    Ok(())
}
