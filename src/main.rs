use std::env;
use std::fs;
use std::path::Path;
use std::vec;
use std::process;
use pad::{Alignment, PadStr};

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
            comments: vec![]
        }
    }
}

fn split_to_slides(contents: &str) -> Vec<Slide> {
    let lines: Vec<&str> = contents.split_terminator("\n").collect();
    let mut slides: Vec<Slide> = vec![];
    let mut current_slide = Slide::new();

    for line in lines {
        let trimmed = line.trim();

        if trimmed.starts_with("#") {

            if current_slide.title.len() > 0 || current_slide.content.len() > 0 {
                slides.push(current_slide);

                // reset "current_slide" value
                current_slide = Slide::new();
            }

            current_slide.title = trimmed.to_owned();

        } else if trimmed.starts_with("<!--") {
            // TODO support multiline comments
            let mut uncomment_line = &trimmed[4..];

            if uncomment_line.ends_with("-->") {
                uncomment_line = &uncomment_line[0..uncomment_line.len() - 3];
            }

            current_slide.comments.push(uncomment_line.to_owned());
        } else {
            current_slide.content.push(trimmed.to_owned())
        }
    }

    if current_slide.title.len() > 0 {
        slides.push(current_slide);
    }

    return slides;
}

pub struct VimSlidesArgs {
    pub source_file: String,
    pub destination: String,
}

fn parse_args(raw_args: Vec<String>) -> VimSlidesArgs {
    println!("args: {}", raw_args.join(" | "));

    let args_len = raw_args.iter().len();

    match args_len {
        0 | 1 => panic!("no arguments provided"),
        2 => VimSlidesArgs {
            source_file: raw_args[1].clone(),
            destination: String::from("./slides"),
        },
        _ => VimSlidesArgs {
            source_file: raw_args[1].clone(),
            destination: raw_args[2].clone(),
        }
    }
}

fn main() -> std::io::Result<()> {
    let args = parse_args(env::args().collect());

    let contents = fs::read_to_string(args.source_file)
        .expect("Something went wrong reading the source file");

    let slides = split_to_slides(contents.as_str());

    let dest_path = Path::new(&args.destination);
    if dest_path.exists() {
        if !dest_path.is_dir() {
            eprintln!("Destination exists, but it is a file, not a directory");
            process::exit(1);
        } else {
            // TODO erase dir contents, prompt user?
            println!("Destination dir already exists");
        }
    } else {
        fs::create_dir(dest_path)?;
        println!("Created directory: {}", args.destination);
    }

    println!("slides qty {}", slides.len());

    for (i, slide) in slides.iter().enumerate() {
        let pad_char = "0".chars().next().unwrap();
        let slide_id = format!("{}", i + 1)
            .to_string()
            .pad(
                3,
                pad_char,
                Alignment::Right,
                true
            );
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
    println!("navigate between slides with :next and :prev");

    Ok(())
}
