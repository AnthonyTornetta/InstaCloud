use std::fs;

use walkdir::WalkDir;

mod api;

fn prepare_gen() {
    // Removes all old generated files w/out deleting the current terraform state.
    let delete_extensions = ["zip", "tf"];
    for entry in WalkDir::new("terraform/generated") {
        let Ok(entry) = entry else {
            continue;
        };

        if entry
            .path()
            .extension()
            .map(|x| delete_extensions.contains(&x.to_str().unwrap_or("")))
            .unwrap_or(false)
        {
            fs::remove_file(entry.path()).expect("Unable to remove old tf file.");
        }
    }

    fs::create_dir_all("terraform/generated").expect("Unable to create generated dir!");
}
