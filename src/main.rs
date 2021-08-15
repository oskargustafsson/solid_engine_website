use dircpy::copy_dir;
use handlebars::Handlebars;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const INDEX_TEMPLATE_FILENAME: &str = "index.html";

struct Page<'a> {
	title: &'static str,
	source_dir: &'a Path,
	target_dir: &'a Path,
	main_content_file: &'a Path,
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

	let index_template_src = fs::read_to_string(source_path.join(INDEX_TEMPLATE_FILENAME))?;

	let mut handlebars = Handlebars::new();
	handlebars.register_template_string(INDEX_TEMPLATE_FILENAME, index_template_src)?;

	for page in pages {
		let main_content_file_path = source_path
			.join(page.source_dir)
			.join(page.main_content_file);
		let main_content = fs::read_to_string(main_content_file_path)?;

		let mut data = BTreeMap::new();
		data.insert("main", main_content);
		data.insert("title", page.title.into());

		let page_target_path = target_path.join(page.target_dir);

		if !Path::exists(&page_target_path) {
			fs::create_dir(page_target_path.clone())?;
		}

		let rendered_page = handlebars.render(INDEX_TEMPLATE_FILENAME, &data)?;

		let mut out_file = File::create(page_target_path.join(INDEX_TEMPLATE_FILENAME))?;
		out_file.write_all(rendered_page.as_bytes())?;
	}

	Ok(())
}
