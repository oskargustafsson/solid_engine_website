use dircpy::copy_dir;
use handlebars::Handlebars;
use notify::{watcher, RecursiveMode::Recursive, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::{env, fs};

const INDEX_TEMPLATE_FILENAME: &str = "index.hbs";
const NAV_TEMPLATE_FILENAME: &str = "nav.hbs";

#[derive(Debug)]
struct Page<'a> {
	title: &'static str,
	source_dir: &'a Path,
	target_dir: &'a Path,
	main_content_file: &'a Path,
}

#[derive(Serialize, Deserialize)]
struct MenuItem {
	text: String,
	path: String,
	is_current: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
	let args: Vec<String> = env::args().collect();

	generate_website()?;

	if 1 < args.len() && args[1] == "--watch" {
		let (tx, rx) = channel();
		let mut watcher = watcher(tx, Duration::from_millis(100))?;
		watcher.watch("website_src", Recursive)?;

		loop {
			match rx.recv() {
				Ok(mut event) => {
					while let Ok(next_event) = rx.try_recv() {
						println!("Ignoring event, due to lack of time {:?}", event);
						event = next_event;
					}
					println!("Regenerating because of {:?}", event);
					generate_website()?;
				}
				Err(e) => {
					return Err(Box::new(e));
				}
			}
		}
	}

	Ok(())
}

fn generate_website() -> Result<(), Box<dyn Error>> {
	let source_path = Path::new("website_src");
	let target_path = Path::new("docs"); // Dir name required by GitHub Pages

	if Path::exists(&target_path) {
		fs::remove_dir_all(target_path)?;
	}
	fs::create_dir(target_path)?;

	{
		let res_path = Path::new("res");
		copy_dir(source_path.join(res_path), target_path.join(res_path))?;
	}

	{
		let root_res_path = Path::new("root_res");
		copy_dir(source_path.join(root_res_path), target_path)?;
	}

	// TODO: Store this information in a data file
	let pages = vec![
		Page {
			title: "Home",
			source_dir: Path::new("home"),
			target_dir: Path::new(""),
			main_content_file: Path::new("main.html"),
		},
		Page {
			title: "Engine",
			source_dir: Path::new("engine"),
			target_dir: Path::new("engine"),
			main_content_file: Path::new("main.html"),
		},
		Page {
			title: "Voxel Editor",
			source_dir: Path::new("voxel_editor"),
			target_dir: Path::new("voxel-editor"),
			main_content_file: Path::new("main.html"),
		},
		Page {
			title: "Ball pointer",
			source_dir: Path::new("ball_pointer"),
			target_dir: Path::new("ball-pointer"),
			main_content_file: Path::new("main.html"),
		},
		Page {
			title: "UI",
			source_dir: Path::new("ui"),
			target_dir: Path::new("ui"),
			main_content_file: Path::new("main.html"),
		},
		Page {
			title: "State",
			source_dir: Path::new("state"),
			target_dir: Path::new("state"),
			main_content_file: Path::new("main.html"),
		},
	];

	let mut handlebars = Handlebars::new();

	for template_filename in [INDEX_TEMPLATE_FILENAME, NAV_TEMPLATE_FILENAME] {
		handlebars
			.register_template_file(template_filename, source_path.join(template_filename))?;
	}

	for page in &pages {
		let menu_items: Vec<MenuItem> = pages
			.iter()
			.map(|p| MenuItem {
				text: p.title.into(),
				path: p.target_dir.to_string_lossy().into(),
				is_current: p.target_dir == page.target_dir,
			})
			.collect();
		let nav_content = handlebars.render(NAV_TEMPLATE_FILENAME, &menu_items)?;

		let main_content_file_path = source_path
			.join(page.source_dir)
			.join(page.main_content_file);
		let main_content = fs::read_to_string(main_content_file_path)?;

		let mut data = BTreeMap::new();
		data.insert("nav_content", nav_content);
		data.insert("main_content", main_content);
		data.insert("title", page.title.into());

		let page_target_path = target_path.join(page.target_dir);

		if !Path::exists(&page_target_path) {
			fs::create_dir(page_target_path.clone())?;
		}

		let mut out_file = File::create(page_target_path.join("index.html"))?;
		handlebars.render_to_write(INDEX_TEMPLATE_FILENAME, &data, &mut out_file)?;
	}

	Ok(())
}
