use directories::ProjectDirs;

pub fn get_project_dir() -> ProjectDirs {
    ProjectDirs::from("io", "kjuulh", "hyperlog").expect("to be able to get project dirs")
}
