use dircpy::copy_dir;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::Path;

const INDEX_TEMPLATE_FILENAME: &str = "index.hbs";
const NAV_TEMPLATE_FILENAME: &str = "nav.hbs";

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
	let source_path = Path::new("website_src");
	let target_path = Path::new("website_target");
	let res_path = Path::new("res");

	if Path::exists(&target_path) {
		fs::remove_dir_all(target_path)?;
	}
	fs::create_dir(target_path)?;
	copy_dir(source_path.join(res_path), target_path.join(res_path))?;

	let pages = vec![
		Page {
			title: "Home",
			source_dir: Path::new("home"),
			target_dir: Path::new(""),
			main_content_file: Path::new("main.html"),
		},
		Page {
			title: "Voxel Editor",
			source_dir: Path::new("voxel_editor"),
			target_dir: Path::new("voxel-editor"),
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
