use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct ViewableFileInfo {
	pub name: String,
	pub path: String,
	pub is_dir: bool,
	pub date_created: SystemTime,
	pub date_last_modified: SystemTime,
	pub size: u64
}