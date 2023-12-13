pub fn get_absolute_filepath(
	filepath: &str) -> String
{
	if
		filepath.len() == 0 ||
		filepath.chars().nth(0).unwrap() == '/' ||
		filepath.split(':').count() > 1
	{
		return filepath.to_string()
	}

	let root_directory: String = String::from(env!("CARGO_MANIFEST_DIR"));

	return format!("{}{}", root_directory, filepath).to_string();
}
