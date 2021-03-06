use chrono::prelude::*;
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
    pub notes: Vec<String>,
}

impl Slide {
    fn new() -> Slide {
        Slide {
            title: "".to_owned(),
            content: vec![],
            notes: vec![],
        }
    }
}

fn pad_num(width: usize, arg: u32) -> String {
    let s = format!("{}", arg);

    s.pad(width, '0', Alignment::Right, true)
}

fn timestamp() -> String {
    let local = Local::now();

    return format!(
        "[{}:{}:{}]",
        pad_num(2, local.hour()),
        pad_num(2, local.minute()),
        pad_num(2, local.second()),
    );
}

fn split_to_slides(contents: &str) -> Vec<Slide> {
    let lines: Vec<&str> = contents.split_terminator('\n').collect();
    let mut slides: Vec<Slide> = vec![];
    let mut current_slide = Slide::new();

    for raw_line in lines {
        let line = raw_line.trim_end();
        let trimmed = line.trim_start();

        if line.trim_start().starts_with('#') {
            if !current_slide.title.is_empty() || !current_slide.content.is_empty() {
                slides.push(current_slide);

                // reset "current_slide" value
                current_slide = Slide::new();
            }

            current_slide.title = trimmed.to_owned();
        } else if let Some(uncomment_line) = trimmed.strip_prefix("<!--") {
            // TODO support multiline comments

            match uncomment_line.strip_suffix("-->") {
                Some(comment_body) => current_slide.notes.push(comment_body.trim().to_owned()),
                None => current_slide.notes.push(uncomment_line.trim().to_owned()),
            };
        } else {
            // preserve original content indentation
            current_slide.content.push(line.to_owned());
        }
    }

    if !current_slide.title.is_empty() {
        slides.push(current_slide);
    }

    slides
}

fn create_slides_from_path(
    source: &str,
    dest: &str,
    notes_path: Option<&str>,
    verbose: bool,
) -> std::io::Result<()> {
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
        let slide_id = format!("{}", i + 1)
            .to_string()
            .pad(3, '0', Alignment::Right, true);
        let slide_filepath = dest_path.join(format!("{}.md", slide_id));

        let lines = slide.content.join("\n");
        let slide_content = format!("{}\n{}", slide.title, lines);

        fs::write(slide_filepath, slide_content.trim()).expect("Could not write file for a slide");
    }

    if let Some(notes_path) = notes_path {
        let notes_title = "# Speaker notes".to_string();
        let empty_line = "".to_string();
        let mut notes_lines: Vec<String> = vec![notes_title, empty_line.clone()];
        for (i, slide) in slides.iter().enumerate() {
            let title = format!("{}(slide {})", slide.title, i + 1);

            notes_lines.push(title);
            notes_lines.push(empty_line.clone());

            if !slide.notes.is_empty() {
                for note in slide.notes.iter() {
                    notes_lines.push(note.to_string());
                    notes_lines.push(empty_line.clone());
                }
            } else {
                notes_lines.push("empty".to_string());
            }
            notes_lines.push(empty_line.clone());
        }

        let notes_content = notes_lines.join("\n");

        fs::write(notes_path, notes_content)?;
    }

    let editor: String = env::var("EDITOR").unwrap_or_else(|_| String::from("vi"));

    let slides_glob = dest_path
        .join("*")
        .into_os_string()
        .into_string()
        .expect("* is a string");

    println!(
        "{} Done, to open slides run: {} {}",
        timestamp(),
        editor,
        slides_glob
    );

    if verbose {
        println!("navigate between slides with :next and :prev");
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
        .arg(
            Arg::with_name("notes")
                .help("path to a file to store speaker notes")
                .takes_value(true)
                .required(false)
                .long("notes"),
        )
        .get_matches();

    let dest_path_str = matches.value_of("destination").unwrap();

    let source_filepath = matches.value_of("source").unwrap();

    let comments_path = matches.value_of("notes");

    create_slides_from_path(source_filepath, dest_path_str, comments_path, true)?;

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
                    create_slides_from_path(source_filepath, dest_path_str, comments_path, false)?;
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    }

    Ok(())
}
