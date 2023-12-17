pub fn get_project_root() -> std::io::Result<std::path::PathBuf> 
{
    let path = std::env::current_dir()?;
    let mut path_ancestors = path.as_path().ancestors();

    while let Some(path) = path_ancestors.next()
    {
        let has_cargo =std::fs::read_dir(path)?
            .into_iter()
            .any(|p| p.unwrap().file_name() == std::ffi::OsString::from("Cargo.lock"));

        if has_cargo 
        {
            return Ok(path.into())
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Ran out of places to find Cargo.toml"))

}