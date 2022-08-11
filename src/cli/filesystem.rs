use std::collections::HashMap;
use std::fs;
use std::io::{ErrorKind, Result as IOResult};

use crate::package::{Composition, Package};
use crate::parser::{composition::handle_composition, package::handle_pkg, SspParser};

use crate::cli::Result;

use super::project::Assumption;

#[allow(clippy::type_complexity)]
pub fn read_directory(dir_path: &str) -> (Vec<(String, String)>, Vec<(String, String)>) {
    let mut pkgs_list = vec![];
    let mut comp_list = vec![];

    let dir_list = fs::read_dir(dir_path).expect("cannot list directory");

    for dir_entry in dir_list {
        let dir_entry = dir_entry.unwrap();
        match dir_entry.file_name().to_str() {
            None => {
                continue;
            }
            Some(name) => {
                if name.ends_with(".ssp") {
                    let filecontent = fs::read_to_string(dir_entry.path());
                    assert!(filecontent.is_ok(), "cannot read file {}", name);

                    let filecontent = filecontent.unwrap().clone();
                    if name.ends_with(".pkg.ssp") {
                        pkgs_list.push((name.to_owned(), filecontent));
                    } else if name.ends_with(".comp.ssp") {
                        comp_list.push((name.to_owned(), filecontent));
                    }
                }
            }
        }
    }

    (pkgs_list, comp_list)
}

pub fn read_packages_directory_or_panic(dir_path: &str) -> Vec<(String, String)> {
    let mut pkgs_list = vec![];

    let dir_list = fs::read_dir(dir_path).expect("cannot list directory");

    for dir_entry in dir_list {
        let dir_entry = dir_entry.unwrap();
        match dir_entry.file_name().to_str() {
            None => {
                continue;
            }
            Some(name) => {
                if name.ends_with(".ssp") {
                    let filecontent = fs::read_to_string(dir_entry.path());
                    assert!(filecontent.is_ok(), "cannot read file {}", name);

                    let filecontent = filecontent.unwrap().clone();
                    if name.ends_with(".pkg.ssp") {
                        pkgs_list.push((name.to_owned(), filecontent));
                    }
                }
            }
        }
    }

    pkgs_list
}

pub fn read_packages_directory(dir_path: &str) -> IOResult<Vec<(String, String)>> {
    let mut pkgs_list = vec![];

    for dir_entry in fs::read_dir(dir_path)? {
        let dir_entry = dir_entry?;
        if let Some(name) = dir_entry.file_name().to_str() {
            if name.ends_with(".pkg.ssp") {
                let filecontent = fs::read_to_string(dir_entry.path())?;
                pkgs_list.push((name.to_owned(), filecontent));
            }
        }
    }

    Ok(pkgs_list)
}

pub fn read_compositions_directory_or_panic(dir_path: &str) -> Vec<(String, String)> {
    let mut comp_list = vec![];

    let dir_list = fs::read_dir(dir_path).expect("cannot list directory");

    for dir_entry in dir_list {
        let dir_entry = dir_entry.unwrap();
        match dir_entry.file_name().to_str() {
            None => {
                continue;
            }
            Some(name) => {
                if name.ends_with(".comp.ssp") {
                    let filecontent = fs::read_to_string(dir_entry.path());
                    assert!(filecontent.is_ok(), "cannot read file {}", name);

                    let filecontent = filecontent.unwrap().clone();
                    comp_list.push((name.to_owned(), filecontent));
                }
            }
        }
    }

    comp_list
}

pub fn read_compositions_directory(dir_path: &str) -> IOResult<Vec<(String, String)>> {
    let mut comp_list = vec![];

    for dir_entry in fs::read_dir(dir_path)? {
        let dir_entry = dir_entry?;
        if let Some(name) = dir_entry.file_name().to_str() {
            if name.ends_with(".comp.ssp") {
                let filecontent = fs::read_to_string(dir_entry.path())?;
                comp_list.push((name.to_owned(), filecontent));
            }
        }
    }

    Ok(comp_list)
}

pub fn read_proofs_directory(dir_path: &str) -> Vec<String> {
    let mut proof_list = vec![];

    let dir_list = fs::read_dir(dir_path).expect("cannot list directory");

    for dir_entry in dir_list {
        let dir_entry = dir_entry.unwrap();
        match dir_entry.file_name().to_str() {
            None => {
                continue;
            }
            Some(name) => {
                let path = dir_entry.path();
                let is_file = dir_entry.metadata().unwrap().is_file();
                let is_toml = is_file && "toml" == path.extension().unwrap();
                let is_stringable = path.as_os_str().to_str().is_some();
                if is_file && is_toml && is_stringable {
                    let proof_name = path.file_stem().unwrap().to_str().unwrap();
                    let mut dir_path = path.clone();
                    dir_path.pop();
                    dir_path.push(proof_name);
                    if dir_path.exists() {
                        proof_list.push(proof_name.into());
                    }
                }
            }
        }
    }

    proof_list
}

pub fn parse_packages_or_panic(
    pkgs_list: &[(String, String)],
) -> (HashMap<String, Package>, HashMap<String, &String>) {
    let pkgs_list: Vec<_> = pkgs_list
        .iter()
        .map(|(filename, contents)| {
            let mut ast = SspParser::parse_package(contents)
                .unwrap_or_else(|e| panic!("error parsing file {}: {:#?}", filename, e));
            let (pkg_name, pkg) = handle_pkg(ast.next().unwrap());
            (filename, contents, ast, pkg_name, pkg)
        })
        .collect();

    let mut pkgs_map = HashMap::new();
    let mut pkgs_filenames = HashMap::new();

    for (filename, _, _, pkg_name, pkg) in pkgs_list {
        if let Some(other_filename) = pkgs_filenames.get(&pkg_name) {
            panic!(
                "Package {:?} redefined in {} (originally defined in {})",
                pkg_name, filename, other_filename
            )
        }

        pkgs_map.insert(pkg_name.clone(), pkg);
        pkgs_filenames.insert(pkg_name, filename);
    }

    (pkgs_map, pkgs_filenames)
}
pub fn parse_packages(
    pkgs_list: &[(String, String)],
) -> Result<(HashMap<String, Package>, HashMap<String, &String>)> {
    let pkgs_list: Vec<_> = pkgs_list
        .iter()
        .map(|(filename, contents)| {
            let mut ast = SspParser::parse_package(contents)?;
            let (pkg_name, pkg) = handle_pkg(ast.next().unwrap());

            Ok((filename, ast, contents, pkg_name, pkg))
        })
        .collect::<Result<_>>()?;

    let mut pkgs_map = HashMap::new();
    let mut pkgs_filenames = HashMap::new();

    for (filename, _, _, pkg_name, pkg) in pkgs_list {
        if let Some(other_filename) = pkgs_filenames.get(&pkg_name) {
            panic!(
                "Package {:?} redefined in {} (originally defined in {})",
                pkg_name, filename, other_filename
            )
        }

        pkgs_map.insert(pkg_name.clone(), pkg);
        pkgs_filenames.insert(pkg_name, filename);
    }

    Ok((pkgs_map, pkgs_filenames))
}

pub fn parse_composition_or_panic(
    comp_list: &[(String, String)],
    pkgs_map: &HashMap<String, Package>,
) -> HashMap<String, Composition> {
    let comp_list: Vec<_> = comp_list
        .iter()
        .map(|(filename, contents)| {
            let mut ast = match SspParser::parse_composition(contents) {
                Ok(ast) => ast,
                Err(e) => {
                    panic!("error parsing file {}: {:#?}", filename, e);
                }
            };

            let comp = handle_composition(ast.next().unwrap(), pkgs_map);
            let comp_name = comp.name.clone();
            (filename, contents, ast, comp_name, comp)
        })
        .collect();

    let mut comp_map = HashMap::new();
    for (_, _, _, comp_name, comp) in comp_list {
        comp_map.insert(comp_name, comp);
    }

    comp_map
}

pub fn parse_composition(
    comp_list: &[(String, String)],
    pkgs_map: &HashMap<String, Package>,
) -> Result<HashMap<String, Composition>> {
    let comp_list: Vec<_> = comp_list
        .iter()
        .map(|(filename, contents)| {
            let mut ast = SspParser::parse_composition(contents)?;
            let comp = handle_composition(ast.next().unwrap(), pkgs_map);
            let comp_name = comp.name.clone();

            Ok((filename, contents, ast, comp_name, comp))
        })
        .collect::<Result<_>>()?;

    let mut comp_map = HashMap::new();
    for (_, _, _, comp_name, comp) in comp_list {
        comp_map.insert(comp_name, comp);
    }

    Ok(comp_map)
}

pub const PROJECT_FILE: &str = "ssp.toml";

pub fn find_project_root() -> IOResult<std::path::PathBuf> {
    let mut dir = std::env::current_dir()?;

    loop {
        let lst = dir.read_dir()?;
        for entry in lst {
            let entry = entry?;
            let file_name = match entry.file_name().into_string() {
                Err(_) => continue,
                Ok(name) => name,
            };
            if file_name == PROJECT_FILE {
                return Ok(dir);
            }
        }

        match dir.parent() {
            None => return Err(std::io::Error::from(ErrorKind::NotFound)),
            Some(parent) => dir = parent.into(),
        }
    }
}

pub fn read_game_hops(dir_path: &str) -> IOResult<Vec<crate::cli::project::GameHop>> {
    unimplemented!();
}

pub fn read_assumptions_directory(dir_path: &str) -> IOResult<Vec<(String, String)>> {
    unimplemented!();
}

pub fn parse_assumptions(pkgs_list: &[(String, String)]) -> Result<HashMap<String, Assumption>> {
    unimplemented!();
}
