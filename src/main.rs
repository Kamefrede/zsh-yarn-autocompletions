mod deps;
mod scripts;

fn main() {
    let mut args = std::env::args();
    match args.next().unwrap_or_default().as_str() {
        "scripts" => print!("{}", scripts::fetch_npm_scripts().unwrap_or_default()),
        "add" => println!(
            "{}",
            deps::return_dependencies(None).unwrap_or_default()
        ),
        "add-dev" => println!(
            "{}",
            deps::return_dev_dependencies(None).unwrap_or_default()
        ),
        "remove" => print!(
            "{}",
            deps::fetch_installed_packages().unwrap_or_default()
        ),
        "why" => print!("{}", deps::list_node_modules().unwrap_or_default()),
        _ => (),
    };
}
