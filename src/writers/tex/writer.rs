use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::expressions::Expression;
use crate::identifier::Identifier;
use crate::package::{Composition, Edge, Export, OracleDef, PackageInstance};
use crate::proof::GameHop;
use crate::proof::Proof;
use crate::statement::{CodeBlock, Statement};
use crate::types::Type;

/// TODO: Move to struct so we can have verbose versions (e.g. writing types to expressions)

fn genindentation(cnt: u8) -> String {
    let mut acc = String::new();
    for _ in 0..cnt {
        acc = format!("{}\\pcind", acc);
    }
    acc
}

struct BlockWriter<'a> {
    file: &'a mut File,
}

impl<'a> BlockWriter<'a> {
    fn new(file: &'a mut File) -> BlockWriter<'a> {
        BlockWriter { file }
    }

    fn ident_to_tex(&self, ident: &Identifier) -> String {
        format!("\\n{{{}}}", ident.ident().replace('_', "\\_"))
    }

	fn type_to_tex(&self, tipe: &Type) -> String {
        format!("\\O{{{:?}}}", tipe)
	}

    fn expression_to_tex(&self, expr: &Expression) -> String {
        match expr {
            Expression::Bot => "\\bot".to_string(),
            Expression::Identifier(ident) => self.ident_to_tex(ident),
            Expression::Not(expr) => format!("\\neg {}", self.expression_to_tex(expr)),
            Expression::Unwrap(expr) => format!("\\O{{unwrap}}({})", self.expression_to_tex(expr)),
            Expression::Some(expr) => format!("\\O{{some}}({})", self.expression_to_tex(expr)),
			Expression::None(tipe) => format!("\\O{{none}}({})", self.type_to_tex(tipe)),
            Expression::Add(lhs, rhs) => format!(
                "({} + {})",
                self.expression_to_tex(lhs),
                self.expression_to_tex(rhs)
            ),
            Expression::TableAccess(ident, expr) => format!(
                "{}[{}]",
                self.ident_to_tex(ident),
                self.expression_to_tex(expr)
            ),
            Expression::Equals(exprs) => exprs
                .iter()
                .map(|expr| self.expression_to_tex(expr))
                .collect::<Vec<_>>()
                .join(" = "),
            Expression::Tuple(exprs) => {
                format!(
                    "({})",
                    exprs
                        .iter()
                        .map(|expr| self.expression_to_tex(expr))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Expression::FnCall(name, args) => {
                format!(
                    "\\O{{{}}}({})",
                    name.ident(),
                    args.iter()
                        .map(|expr| self.expression_to_tex(expr))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            _ => {
                format!("{:?}", expr)
            }
        }
    }

    fn write_statement(&mut self, statement: &Statement, indentation: u8) -> std::io::Result<()> {
        match &statement {
            Statement::Abort(_) => {
                writeln!(self.file, "{} \\pcabort\\\\", genindentation(indentation))?;
            }
            Statement::Return(None, _) => {
                writeln!(self.file, "{} \\pcreturn\\\\", genindentation(indentation))?;
            }
            Statement::Return(Some(expr), _) => {
                writeln!(
                    self.file,
                    "{} \\pcreturn {}\\\\",
                    genindentation(indentation),
                    self.expression_to_tex(expr)
                )?;
            }
            Statement::Assign(ident, None, expr, _) => {
                writeln!(
                    self.file,
                    "{} {} \\gets {}\\\\",
                    genindentation(indentation),
                    self.ident_to_tex(ident),
                    self.expression_to_tex(expr)
                )?;
            }
            Statement::Assign(ident, Some(idxexpr), expr, _) => {
                writeln!(
                    self.file,
                    "{} {}[{}] \\gets {}\\\\",
                    genindentation(indentation),
                    self.ident_to_tex(ident),
                    self.expression_to_tex(idxexpr),
                    self.expression_to_tex(expr)
                )?;
            }
            Statement::Parse(ids, expr, _) => {
                writeln!(
                    self.file,
                    "{}\\pcparse {} \\pcas {}\\\\",
                    genindentation(indentation),
                    self.expression_to_tex(expr),
                    ids.iter()
                        .map(|ident| self.ident_to_tex(ident))
                        .collect::<Vec<_>>()
                        .join(", ")
                )?;
            }
            Statement::IfThenElse(expr, ifcode, elsecode, _) => {
                writeln!(
                    self.file,
                    "{}\\pcif {} \\pcthen\\\\",
                    genindentation(indentation),
                    self.expression_to_tex(expr)
                )?;
                self.write_codeblock(ifcode, indentation + 1)?;
                if !elsecode.0.is_empty() {
                    writeln!(self.file, "{}\\pcelse\\\\", genindentation(indentation))?;
                    self.write_codeblock(elsecode, indentation + 1)?;
                }
            }
            Statement::For(_, _, _, _, _) => todo!(),
            Statement::Sample(ident, None, maybecnt, tipe, _) => {
                let cnt = maybecnt.expect("Expected samplified input");

                writeln!(
                    self.file,
                    "{}{} \\stackrel{{{}}}{{\\sample}} {}\\\\",
                    genindentation(indentation),
                    self.ident_to_tex(ident),
                    cnt,
                    self.type_to_tex(tipe)
                )?;
            }
            Statement::Sample(ident, Some(idxexpr), maybecnt, tipe, _) => {
                let cnt = maybecnt.expect("Expected samplified input");

                writeln!(
                    self.file,
                    "{}{}[{}] \\stackrel{{{}}}{{\\samples}} {}\\\\",
                    genindentation(indentation),
                    self.ident_to_tex(ident),
                    self.expression_to_tex(idxexpr),
                    cnt,
                    self.type_to_tex(tipe)
                )?;
            }
            Statement::InvokeOracle {
                id: ident,
                opt_idx: None,
                name,
                args,
                target_inst_name: Some(target_inst_name),
                tipe: _,
                ..
            } => {
                writeln!(self.file,
                         "{}{} \\stackrel{{\\mathsf{{\\tiny{{invoke}}}}}}{{\\gets}} \\O{{{}}}({}) \\pccomment{{Pkg: {}}} \\\\",
                         genindentation(indentation),
                         self.ident_to_tex(ident), name,
                         args.iter().map(|expr| self.expression_to_tex(expr)).collect::<Vec<_>>().join(", "),
                         target_inst_name.replace('_',"\\_")
                )?;
            }
            Statement::InvokeOracle {
                id: ident,
                opt_idx: Some(idxexpr),
                name,
                args,
                target_inst_name: Some(target_inst_name),
                tipe: _,
                ..
            } => {
                writeln!(self.file,
                         "{}{}[{}] \\stackrel{{\\mathsf{{\\tiny invoke}}}}{{\\gets}} \\O{{{}}}({}) \\pccomment{{Pkg: {}}} \\\\",
                         genindentation(indentation),
                         self.ident_to_tex(ident),
                         self.expression_to_tex(idxexpr),
                         name,
                         args.iter().map(|expr| self.expression_to_tex(expr)).collect::<Vec<_>>().join(", "),
                         target_inst_name.replace('_',"\\_")
                )?;
            }
            Statement::InvokeOracle {
                target_inst_name: None,
                ..
            } => {
                unreachable!("Expect oracle-lowlevelified input")
            }
        }
        Ok(())
    }

    fn write_codeblock(&mut self, codeblock: &CodeBlock, indentation: u8) -> std::io::Result<()> {
        for stmt in &codeblock.0 {
            self.write_statement(stmt, indentation)?
        }
        Ok(())
    }
}

pub fn tex_write_oracle(
    oracle: &OracleDef,
    pkgname: &str,
    target: &Path,
) -> std::io::Result<String> {
    let fname = target.join(format!("Oracle_{}_{}.tex", pkgname, oracle.sig.name));
    let mut file = File::create(fname.clone())?;
    let mut writer = BlockWriter::new(&mut file);

    writeln!(
        writer.file,
        "\\procedure{{\\O{{{}}}({})}}{{",
        oracle.sig.name,
        oracle
            .sig
            .args
            .iter()
            .map(|(a, _)| a.clone())
            .collect::<Vec<_>>()
            .join(", ")
    )?;

    let codeblock = &oracle.code;
    writer.write_codeblock(codeblock, 0)?;

    writeln!(writer.file, "}}")?;
    Ok(fname.to_str().unwrap().to_string())
}

pub fn tex_write_package(package: &PackageInstance, target: &Path) -> std::io::Result<String> {
    let fname = target.join(format!("Package_{}.tex", package.name));
    let mut file = File::create(fname.clone())?;

    writeln!(
        file,
        "\\begin{{pcvstack}}\\underline{{\\underline{{\\M{{{}}}}}}}\\\\\\begin{{pchstack}}",
        package.name.replace('_', "\\_")
    )?;

    for oracle in &package.pkg.oracles {
        let oraclefname = tex_write_oracle(oracle, &package.name, target)?;
        writeln!(file, "\\input{{{}}}\\pchspace", oraclefname)?;
    }
    writeln!(file, "\\end{{pchstack}}\\end{{pcvstack}}")?;

    Ok(fname.to_str().unwrap().to_string())
}

fn tex_write_document_header(mut file: &File) -> std::io::Result<()> {
    writeln!(file, "\\documentclass[a4paper,landscape]{{article}}")?;
    writeln!(file, "\\usepackage[margin=1in]{{geometry}}")?;
    writeln!(file, "\\usepackage[operators]{{cryptocode}}")?;
    writeln!(file, "\\usepackage{{tikz}}")?;
    writeln!(
        file,
        "\\renewcommand\\O[1]{{\\ensuremath{{\\mathsf{{#1}}}}}}"
    )?;
    writeln!(
        file,
        "\\newcommand{{\\M}}[1]{{\\ensuremath{{\\text{{\\texttt{{#1}}}}}}}}"
    )?;
    writeln!(
        file,
        "\\newcommand{{\\n}}[1]{{\\ensuremath{{\\mathit{{#1}}}}}}"
    )?;
    writeln!(file, "\\tikzstyle{{package}} = [inner sep=1pt,align=center,rounded corners,draw,minimum width=2cm,minimum height=1cm,font=\\small]")?;
    writeln!(file, "\\tikzstyle{{onarrow}} = [inner sep=1pt,font=\\scriptsize,anchor=east,at end,xshift=-0.1mm,align=left,fill=white]")?;
    Ok(())
}

fn tex_write_composition_graph(
    mut file: &File,
    composition: &Composition,
    pkgmap: &[(std::string::String, std::string::String)],
) -> std::io::Result<()> {
    let mut printed = Vec::new();
    let mut newly = Vec::new();

    let mut tikzx = 0;
    let mut tikzy = 0;

    writeln!(file, "\\begin{{tikzpicture}}")?;
    while printed.len() < composition.pkgs.len() {
        for i in 0..composition.pkgs.len() {
            if printed.contains(&i) {
                continue;
            }

            if !composition
                .edges
                .iter()
                .any(|Edge(from, to, _oracle)| i == *from && !printed.contains(to))
            {
                let fill = if pkgmap
                    .iter()
                    .find(|(pkgname, _)| composition.pkgs[i].name == *pkgname)
                    .is_some()
                {
                    "red!50"
                } else {
                    "white"
                };

                writeln!(
                    file,
                    "\\node[align=center,package,fill={fill}] (node{}) at ({}, {}) {{\\M{{{}}}\\\\\\M{{{}}}}};",
                    i,
                    tikzx,
                    tikzy,
                    composition.pkgs[i].name.replace('_', "\\_"),
                    composition.pkgs[i].pkg.name.replace('_', "\\_")
                )?;
                newly.push(i);
                tikzy -= 2;

                for Edge(from, to, oracle) in &composition.edges {
                    if i == *from {
                        writeln!(file, "\\draw[-latex,rounded corners] (node{}) -- ($(node{}.east) + (1,0)$) |- node[onarrow] {{\\O{{{}}}}} (node{});", from, from, oracle.name, to)?;
                    }
                }
            }
        }
        printed.append(&mut newly);
        tikzx -= 4;
        tikzy = tikzx / 4;
    }

    writeln!(
        file,
        "\\node[package] (nodea) at ({}, {}) {{$A$}};",
        tikzx, tikzy
    )?;
    for Export(to, oracle) in &composition.exports {
        writeln!(file, "\\draw[-latex,rounded corners] (nodea) -- ($(nodea.east) + (1,0)$) |- node[onarrow] {{\\O{{{}}}}} (node{});", oracle.name, to)?;
    }
    writeln!(file, "\\end{{tikzpicture}}")?;

    Ok(())
}

fn tex_write_composition_graph_file(
    composition: &Composition,
    name: &str,
    target: &Path,
) -> std::io::Result<String> {
    let fname = target.join(format!("CompositionGraph_{}.tex", name));
    let mut file = File::create(fname.clone())?;

    tex_write_composition_graph(&file, composition, &Vec::new())?;

    Ok(fname.to_str().unwrap().to_string())
}

pub fn tex_write_composition(
    composition: &Composition,
    name: &str,
    target: &Path,
) -> std::io::Result<()> {
    let fname = target.join(format!("Composition_{}.tex", name));
    let mut file = File::create(fname)?;

    tex_write_document_header(&file)?;

    writeln!(file, "\\title{{{} Game}}", name)?;
    writeln!(file, "\\begin{{document}}")?;
    writeln!(file, "\\maketitle")?;

    let graphfname = tex_write_composition_graph_file(composition, name, target)?;
	writeln!(file, "\\begin{{center}}")?;
    writeln!(file, "\\input{{{}}}", graphfname)?;
	writeln!(file, "\\end{{center}}")?;

    for pkg in &composition.pkgs {
        let pkgfname = tex_write_package(pkg, target)?;
		writeln!(file, "\\begin{{center}}")?;
        writeln!(file, "\\input{{{}}}", pkgfname)?;
		writeln!(file, "\\end{{center}}")?;
    }

    writeln!(file, "\\end{{document}}")?;

    Ok(())
}

pub fn tex_write_proof(proof: &Proof, name: &str, target: &Path) -> std::io::Result<()> {
    let fname = target.join(format!("Proof_{}.tex", name));
    let mut file = File::create(fname)?;

    tex_write_document_header(&file)?;

    writeln!(file, "\\title{{Proof: {}}}", name)?;
    writeln!(file, "\\begin{{document}}")?;
    writeln!(file, "\\maketitle")?;

    writeln!(file, "\\section{{Games}}")?;

    for instance in &proof.instances {
        writeln!(file, "\\subsection{{{} Game}}", instance.name())?;

        let graphfname = target.join(format!("CompositionGraph_{}.tex", instance.name()));
		writeln!(file, "\\begin{{center}}")?;
        writeln!(file, "\\input{{{}}}", graphfname.display())?;
		writeln!(file, "\\end{{center}}")?;

        for package in &instance.game().pkgs {
            let pkgfname = target.join(format!("Package_{}.tex", package.name));
			writeln!(file, "\\begin{{center}}")?;
            writeln!(file, "\\input{{{}}}", pkgfname.display())?;
			writeln!(file, "\\end{{center}}")?;
        }
    }

    for game_hop in &proof.game_hops {
        match &game_hop {
            GameHop::Reduction(red) => {
                writeln!(file, "\\section{{Reduction to {}}}", red.assumption_name())?;

                writeln!(
                    file,
                    "\\subsection{{Game {} with Assumption Game {} highlighted in red}}",
                    red.left().as_game_inst_name(),
                    red.left().as_assumption_game_inst_name()
                )?;
				writeln!(file, "\\begin{{center}}")?;
                let left_game_instance = proof
                    .instances
                    .iter()
                    .find(|instance| instance.name() == red.left().as_game_inst_name())
                    .unwrap();
                tex_write_composition_graph(
                    &file,
                    left_game_instance.game(),
                    red.left().pkg_maps(),
                )?;
				writeln!(file, "\\end{{center}}")?;

                writeln!(
                    file,
                    "\\subsection{{Game {} with Assumption Game {} highlighted  in red}}",
                    red.right().as_game_inst_name(),
                    red.right().as_assumption_game_inst_name()
                )?;
				writeln!(file, "\\begin{{center}}")?;
                let right_game_instance = proof
                    .instances
                    .iter()
                    .find(|instance| instance.name() == red.right().as_game_inst_name())
                    .unwrap();
                tex_write_composition_graph(
                    &file,
                    right_game_instance.game(),
                    red.right().pkg_maps(),
                )?;
				writeln!(file, "\\end{{center}}")?;
            }
            GameHop::Equivalence(equiv) => {
                writeln!(
                    file,
                    "\\section{{Equivalence between {} and {}}}",
                    equiv.left_name(),
                    equiv.right_name()
                )?;
            }
        }
    }

    writeln!(file, "\\end{{document}}")?;
    Ok(())
}
