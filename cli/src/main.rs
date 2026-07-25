use lumi_protocol::LumiPackage;
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "new" => {
            if args.len() < 3 {
                println!("Error: Missing site name. Usage: lumi new <site_name>");
                return Ok(());
            }
            if let Err(e) = create_new_site(&args[2]) {
                eprintln!("Error creating new site: {}", e);
                std::process::exit(1);
            }
        }
        "pack" => {
            if args.len() < 4 {
                println!("Error: Usage: lumi pack <site_directory> <output.lpkg>");
                return Ok(());
            }
            if let Err(e) = pack_site(&args[2], &args[3]) {
                eprintln!("Error packing site: {}", e);
                std::process::exit(1);
            }
        }
        _ => print_usage(),
    }
    Ok(())
}

fn print_usage() {
    println!("Lumi Developer SDK CLI v0.2");
    println!("Usage:");
    println!("  lumi new <site_name>                  Scaffold a new LumiML website");
    println!("  lumi pack <site_dir> <output.lpkg>     Compile site into .lpkg archive package");
}

fn create_new_site(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let site_path = Path::new(name);
    if site_path.exists() {
        println!("Error: Directory '{}' already exists.", name);
        return Ok(());
    }

    fs::create_dir_all(site_path.join("assets"))?;

    let manifest_content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
entry = "index.lml"
"#,
        name
    );

    let index_content = format!(
        r#"page {{
    title "{}"

    badge {{
        text "Powered by Lumi Ecosystem v0.2"
    }}

    heading {{
        text "Welcome to {}"
    }}

    paragraph {{
        text "This is your brand new privacy-first website built with LumiML."
    }}

    divider {{}}

    button {{
        text "Explore Docs"
        goto "lumi://docs.lumi"
    }}
}}
"#,
        name, name
    );

    fs::write(site_path.join("manifest.toml"), manifest_content)?;
    fs::write(site_path.join("index.lml"), index_content)?;

    println!("✨ Successfully scaffolded Lumi site '{}'!", name);
    println!("📁 Directory created at: ./{}/", name);
    println!(
        "👉 Run 'lumi pack {} {}.lpkg' to bundle site for lumid server.",
        name, name
    );
    Ok(())
}

fn pack_site(site_dir: &str, output_pkg: &str) -> Result<(), Box<dyn std::error::Error>> {
    let dir_path = Path::new(site_dir);
    if !dir_path.exists() {
        println!("Error: Directory '{}' not found.", site_dir);
        return Ok(());
    }

    let index_file = dir_path.join("index.lml");
    if !index_file.exists() {
        println!("Error: Missing 'index.lml' in '{}'.", site_dir);
        return Ok(());
    }

    let index_lml = fs::read_to_string(index_file)?;
    let pkg = LumiPackage::new_lml(site_dir, &index_lml);

    let bytes = pkg.to_bytes().unwrap_or_default();
    let byte_len = bytes.len();
    fs::write(output_pkg, bytes)?;

    println!(
        "📦 Successfully packed '{}' into '{}' ({} bytes)",
        site_dir, output_pkg, byte_len
    );
    Ok(())
}
