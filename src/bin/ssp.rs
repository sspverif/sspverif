use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use sspds::types::Type;

use clap::{Parser, Subcommand};

use sspds::{
    cli::filesystem::{
        parse_composition_or_panic as parse_composition, parse_packages_or_panic as parse_packages,
        read_directory,
    },
    hacks,
    package::{Composition, Edge, Export},
    writers::{
        pseudocode::writer::Writer, smt::writer::CompositionSmtWriter,
        tex::writer::tex_write_composition,
    },
};

/*

What is the functionality of the tool:
- game hops
  - count epsilons
- assumption
- game description (= composition)
- package
- proof types:
  - code equality
  - assumption application
  - hybrid argument (n-wise parallel composition)
- compute and verify proofs
- generate latex + proof viewer

What do we need for this:
- internal representation of multi-step/multi-game-hop proofs
  - type-checking + rewriting rather than smt
    - rewriting based on assumption or code equivalence
    - user says what rewrite rule to use
- would be nice to pin down directory structure a bit
  - this sort of reflects the data structure for the entire proof, though

----

UI consists of two elements:
- CLI
- Directory Structure

----

$ ssp prove [-u] [real [ideal]]	# prove that games real and ideal are indistinguishable and show epsilons
                                 # also generates proofviewer html
                                # what happens with bugs in the proof?
                                #   each gamehop is a lemma/claim, and when we can't prove it, it can be flagged
                                # when -u is given, automatically re-prove and re-generate proof viewer, when a file is updated. also host a web-server to host proof viewer and give info on syntax errors (-u is for update. maybe w for watch?)
$ ssp check-syntax [pkg]	    # syntax check individual packages
$ ssp gen-latex                 # check syntax and generate latex cryptocode for packages

----

project/
    ssp.toml  -- this is at least needed to indicate that this is the project root
    _build/ 	-- only generated, only read by the user
        code_eq/
            mapping_ideal
        proof_viewer/
            index.html
            ...
        latex/
            packages/
                ...
            graphs/
                ...
    packages/
        key.pkg.ssp
        prf.pkg.ssp
    assumptions/
        prf/
            real.comp.ssp
            ideal.comp.ssp
            // hard-code these names? allow overrides?
            //     maybe for now allow all names
            // type checker needs to validate that interfaces match up
            // - oracles
            // - parameters
            // - how to deal with security parameter?
            // - more?
    games/
        real.comp.ssp
        g1.comp.ssp
        g2.comp.ssp
        ...
        mapping.comp.ssp
        ideal.comp.ssp
    game_hops.toml
        // rough description:
        // real->g1: assumption
        // g1->g2: assumption
        // ...
        // mapping->ideal: code equivalence with code_eq/mapping-ideal_invariant.smt
    code_eq/
        mapping-ideal_invariant.smt

----

game_hops.toml:

This would be the contents is JSONy notation. We'll see how that looks like in toml.
[
    {
        Reduction: {
            left: PackageName
            right: PackageName
            assumption: AssumptionName
            optional:
                name: String
        }
    },
    {
        Equivalence: {
            left: PackageName
            right: PackageName
            invariant: InvariantPath
            optional:
                name: String
        }
    },
    ..
]

*/

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Verifies the code of packages
    Check {
        name: String,
    },

    /// Generates SMT
    Smt {
        name: String,
    },

    /// Generate latex (cryptocode) files
    Latex(LaTeX),

    /// Generate graph representation of the composition
    Graph(Graph),

    /// Give information about the provided code
    Explain(Explain),

    Prove,
}

//#[derive(Parser)]
#[derive(clap::Args)]
#[clap(author, version, about, long_about = None)]
struct ProofCommand {
    #[clap(subcommand)]
    command: Proof,
}

#[derive(Subcommand)]
enum Proof {
    Build { proofname: String },
}

#[derive(clap::Args)]
#[clap(author, version, about, long_about = None)]
struct Explain {
    dirname: String,
    #[clap(short, long)]
    output: Option<String>,
}

#[derive(clap::Args)]
#[clap(author, version, about, long_about = None)]
struct Graph {
    dirname: String,
    #[clap(short, long)]
    output: String,
}

#[derive(clap::Args)]
#[clap(author, version, about, long_about = None)]
struct LaTeX {
    dirname: String,
    #[clap(short, long)]
    output: String,
}

fn graph(args: &Graph) {
    let (pkgs_list, comp_list) = read_directory(&args.dirname);
    let (pkgs_map, _pkgs_filenames) = parse_packages(&pkgs_list);
    let comp_map = parse_composition(&comp_list, &pkgs_map);

    for (name, comp) in comp_map {
        let mut file = File::create(format!("{}/{}.dot", args.output, name)).unwrap();
        writeln!(
            &mut file,
            "digraph test {{\n  rankdir=LR;\n  node [shape=\"box\"];\n"
        )
        .unwrap();

        let Composition {
            pkgs,
            edges,
            exports,
            name: _,
            ..
        } = comp;
        for Edge(source, target, sig) in edges {
            writeln!(
                &mut file,
                "  {} -> {} [label=\"{}\"]",
                pkgs[source].name,
                pkgs[target].name,
                format!("{}", sig).replace('"', "\\\"")
            )
            .unwrap();
        }

        for Export(target, sig) in exports {
            writeln!(
                &mut file,
                "  adversary -> {} [label=\"{}\"]",
                pkgs[target].name,
                format!("{}", sig).replace('"', "\\\"")
            )
            .unwrap();
        }

        writeln!(file, "}}").unwrap();
        println!("Wrote {} to {}/{}.dot", name, args.output, name);
    }
}

fn smt(name: &str) {
    let (pkgs_list, comp_list) = read_directory(name);
    let (pkgs_map, _pkgs_filenames) = parse_packages(&pkgs_list);
    let comp_map = parse_composition(&comp_list, &pkgs_map);
    let mut declared_types = HashSet::new();

    let mut w = std::io::stdout();

    write!(w, "{}", hacks::MaybeDeclaration).unwrap();
    write!(w, "{}", hacks::TuplesDeclaration(2..3)).unwrap();

    let mut names : Vec<_> = comp_map.keys().collect();
    names.sort();

    for name in names {
        let comp = &comp_map[name];
        println!("; {}", name);

        //println!("{:#?}", comp);
        let (comp, scope, _, samp) = match sspds::transforms::transform_all(&comp) {
            Ok(x) => x,
            Err(e) => {
                panic!("found an error in composition {}: {:?}", name, e)
            }
        };
        let mut newtypes : Vec<_> = scope
            .known_types()
            .difference(&declared_types)
            .cloned()
            .collect();
        newtypes.sort();
        for tipe in newtypes {
            match tipe {
                Type::Bits(len) => {
                    println!("(declare-sort Bits_{} 0)", len);
                    println!("(declare-const zero_bits_{} Bits_{})", len, len);
                }
                _ => {}
            }
        }
        declared_types.extend(scope.known_types());

        let mut writer = CompositionSmtWriter::new(&comp, &samp);
        for line in writer.smt_composition_all() {
            write!(&mut std::io::stdout(), "{line}").unwrap();
            //line.fmt(&mut std::io::stdout()).unwrap();
        }
    }

    println!("(check-sat)");
}

fn check(args: &str) {
    let (pkgs_list, comp_list) = read_directory(args);
    let (pkgs_map, _pkgs_filenames) = parse_packages(&pkgs_list);
    let comp_map = parse_composition(&comp_list, &pkgs_map);
    println!("Found {} Compositions", comp_map.len());
    for (name, comp) in comp_map {
        print!("Verifying Composition: {}", name);

        match sspds::transforms::transform_all(&comp) {
            Ok(_) => print!(": OK\n"),
            Err(e) => print!(": FAIL {:?}", e),
        }
    }
}

fn latex(args: &LaTeX) {
    let (pkgs_list, comp_list) = read_directory(&args.dirname);
    let (pkgs_map, _pkgs_filenames) = parse_packages(&pkgs_list);
    let comp_map = parse_composition(&comp_list, &pkgs_map);

    for (name, comp) in comp_map {
        let (comp, _, _) = match sspds::transforms::transform_explain(&comp) {
            Ok(x) => x,
            Err(e) => {
                panic!("found an error in composition {}: {:?}", name, e)
            }
        };

        println!("{}", name);
        tex_write_composition(&comp, &name, Path::new(&args.output));
    }
}

use sspds::project;

fn prove() {
    let proj = project::Project::load().unwrap();
    proj.prove().unwrap();
}

fn explain(args: &Explain) {
    let (pkgs_list, comp_list) = read_directory(&args.dirname);
    let (pkgs_map, _pkgs_filenames) = parse_packages(&pkgs_list);
    let comp_map = parse_composition(&comp_list, &pkgs_map);

    let mut w = Writer::new(std::io::stdout());

    for (name, comp) in comp_map {
        let (comp, _, _) = match sspds::transforms::transform_explain(&comp) {
            Ok(x) => x,
            Err(e) => {
                panic!("found an error in composition {}: {:?}", name, e)
            }
        };

        println!("{}", name);
        for inst in comp.pkgs {
            let pkg = inst.pkg;
            w.write_package(&pkg).unwrap();
        }

        //tex_write_composition(&comp, Path::new(&args.output));
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Check { name } => check(name),
        Commands::Smt { name } => smt(name),
        Commands::Latex(args) => latex(args),
        Commands::Graph(args) => graph(args),
        Commands::Explain(args) => explain(args),
        Commands::Prove => prove(),
    }
}
