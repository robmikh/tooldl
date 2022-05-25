use std::{fs::File, path::Path};

use zip::ZipArchive;

pub fn extract_to_directory<P: AsRef<Path>>(
    file: File,
    path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = ZipArchive::new(file)?;
    let root_path = path.as_ref().to_owned();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let output_path = match file.enclosed_name() {
            Some(path) => {
                let mut output_path = root_path.clone();
                output_path.push(path);
                output_path
            }
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            if !output_path.exists() {
                std::fs::create_dir_all(&output_path)?;
            }
        } else {
            if let Some(parent) = output_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            let mut output_file = File::create(&output_path)?;
            std::io::copy(&mut file, &mut output_file)?;
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&output_path, std::fs::Permissions::from_mode(mode))
                    .unwrap();
            }
        }
    }
    Ok(())
}
