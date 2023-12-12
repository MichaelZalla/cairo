pub fn get_absolute_filepath(
	filepath: &str) -> String
{
	let root_directory: String = String::from(env!("CARGO_MANIFEST_DIR"));

	return format!("{}{}", root_directory, filepath).to_string();
}
