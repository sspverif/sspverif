use std::{fs::File, io::Write};

use crate::{
    expressions::Expression,
    identifier::pkg_ident::PackageIdentifier,
    identifier::pkg_ident::PackageOracleCodeLoopVarIdentifier,
    identifier::Identifier,
    parser::ast::Identifier as _,
    parser::package::ForComp,
    statement::{CodeBlock, InvokeOracleStatement, Statement},
    types::{CountSpec, Type},
};

fn genindentation(cnt: u8) -> String {
    let mut acc = String::new();
    for _ in 0..cnt {
        acc = format!("{acc}\\pcind");
    }
    acc
}

pub(crate) struct BlockWriter<'a> {
    file: &'a mut File,
    lossy: bool,
}

impl<'a> BlockWriter<'a> {
    pub(crate) fn new(file: &'a mut File, lossy: bool) -> BlockWriter<'a> {
        BlockWriter { file, lossy }
    }

    pub(crate) fn file(&self) -> &File {
        self.file
    }

    fn ident_to_tex(&self, ident: &Identifier) -> String {
        format!("\\n{{{}}}", ident.ident().replace('_', "\\_"))
    }

    fn countspec_to_tex(&self, count_spec: &CountSpec) -> String {
        match count_spec {
            CountSpec::Identifier(identifier) => self.ident_to_tex(identifier),
            CountSpec::Literal(num) => format!("{num}"),
            CountSpec::Any => "*".to_string(),
        }
    }

    fn type_to_tex(&self, tipe: &Type) -> String {
        match tipe {
            Type::Bits(n) => format!("\\bin^{{{}}}", self.countspec_to_tex(n)),
            _ => format!("\\O{{{tipe:?}}}"),
        }
    }

    fn type_to_tex_short(&self, tipe: &Type) -> String {
        match tipe {
            Type::Tuple(_) => "\\O{Tuple[..]}".to_string(),
            Type::Bits(n) => format!("\\bin^{{{}}}", self.countspec_to_tex(n)),
            _ => format!("\\O{{{tipe:?}}}"),
        }
    }
    fn forcomp_to_tex(&self, forcomp: &ForComp) -> String {
        match forcomp {
            ForComp::Lt => "<",
            ForComp::Lte => "\\leq",
        }
        .to_string()
    }

    fn logic_to_matrix(&self, join: &str, list: &[String]) -> String {
        assert!(list.len() > 1);
        let trivial = list.join(join);
        if trivial.len() < 50 {
            trivial
        } else {
            let mut it = list.iter();
            let mut lines = vec![format!("\\phantom{{{}}}{}", join, it.next().unwrap())];
            let mut rest: Vec<_> = it.map(|s| format!("{join}{s}")).collect();
            lines.append(&mut rest);
            format!(
                "\\begin{{array}}{{c}}{}\\end{{array}}",
                lines.join("\\pclb")
            )
        }
    }

    fn list_to_matrix(&self, list: &[String]) -> String {
        let mut it = list.iter();
        let mut lines = Vec::new();
        let mut line = Vec::new();
        let mut len = 0;
        loop {
            // maybe this should be a minipage and latex figures linesbreaking ...
            match it.next() {
                None => {
                    if !line.is_empty() {
                        lines.push(line.join(", "));
                    }
                    break;
                }
                Some(s) => {
                    if len + s.len() > 20 {
                        line.push(String::new());
                        lines.push(line.join(", "));
                        line = Vec::new();
                        len = 0;
                    }
                    line.push(s.clone());
                    len = len + std::cmp::max(6, s.len()) - 4 // latex makes string length and text length quite different
                }
            }
        }
        format!(
            "\\begin{{array}}{{c}}{}\\end{{array}}",
            lines.join("\\pclb")
        )
    }

    fn expression_to_tex(&self, expr: &Expression) -> String {
        match expr {
            Expression::Bot => "\\bot".to_string(),
            Expression::IntegerLiteral(val) => format!("{val}"),
            Expression::BooleanLiteral(val) => format!("\\lit{{{val}}}"),
            Expression::Identifier(ident) => self.ident_to_tex(ident),
            Expression::Not(expr) => format!("\\neg {}", self.expression_to_tex(expr)),
            Expression::Unwrap(expr) => {
                if self.lossy {
                    self.expression_to_tex(expr)
                } else {
                    format!(
                        "\\O{{unwrap}}\\left({}\\right)",
                        self.expression_to_tex(expr)
                    )
                }
            }
            Expression::Some(expr) => {
                if self.lossy {
                    self.expression_to_tex(expr)
                } else {
                    format!("\\O{{some}}\\left({}\\right)", self.expression_to_tex(expr))
                }
            }
            Expression::None(tipe) => {
                if self.lossy {
                    "\\bot".to_string()
                } else {
                    format!("\\O{{none}}\\left({}\\right)", self.type_to_tex_short(tipe))
                }
            }
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
            Expression::Or(exprs) => format!(
                "\\left({}\\right)",
                self.logic_to_matrix(
                    " \\vee ",
                    &exprs
                        .iter()
                        .map(|expr| self.expression_to_tex(expr))
                        .collect::<Vec<_>>()
                )
            ),
            Expression::And(exprs) => format!(
                "\\left({}\\right)",
                self.logic_to_matrix(
                    " \\wedge ",
                    &exprs
                        .iter()
                        .map(|expr| self.expression_to_tex(expr))
                        .collect::<Vec<_>>()
                )
            ),
            Expression::Tuple(exprs) => {
                format!(
                    "\\left({}\\right)",
                    self.list_to_matrix(
                        &exprs
                            .iter()
                            .map(|expr| self.expression_to_tex(expr))
                            .collect::<Vec<_>>()
                    )
                )
            }
            Expression::FnCall(name, args) => {
                format!(
                    "\\O{{{}}}({})",
                    self.ident_to_tex(name),
                    args.iter()
                        .map(|expr| self.expression_to_tex(expr))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            _ => {
                format!("{expr:?}")
            }
        }
    }

    fn write_statement(&self, statement: &Statement, indentation: u8) -> String {
        match &statement {
            Statement::Abort(_) => {
                format!("{} \\pcabort\\\\", genindentation(indentation))
            }
            Statement::Return(None, _) => {
                format!("{} \\pcreturn\\\\", genindentation(indentation))
            }
            Statement::Return(Some(expr), _) => {
                format!(
                    "{} \\pcreturn {}\\\\",
                    genindentation(indentation),
                    self.expression_to_tex(expr)
                )
            }
            Statement::Assign(ident, None, expr, _) => {
                format!(
                    "{} {} \\gets {}\\\\",
                    genindentation(indentation),
                    self.ident_to_tex(ident),
                    self.expression_to_tex(expr)
                )
            }
            Statement::Assign(ident, Some(idxexpr), expr, _) => {
                format!(
                    "{} {}[{}] \\gets {}\\\\",
                    genindentation(indentation),
                    self.ident_to_tex(ident),
                    self.expression_to_tex(idxexpr),
                    self.expression_to_tex(expr)
                )
            }
            Statement::Parse(ids, expr, _) => {
                format!(
                    "{}\\pcparse {} \\pcas \\left({}\\right)\\\\",
                    genindentation(indentation),
                    self.expression_to_tex(expr),
                    self.list_to_matrix(
                        &ids.iter()
                            .map(|ident| self.ident_to_tex(ident))
                            .collect::<Vec<_>>()
                    )
                )
            }
            Statement::IfThenElse(ite) => {
                if ite.then_block.0.is_empty()
                    && ite.else_block.0.len() == 1
                    && matches!(ite.else_block.0[0], Statement::Abort(_))
                {
                    // Special Case for asserts
                    format!(
                        "{}\\pcassert {} \\\\",
                        genindentation(indentation),
                        self.expression_to_tex(&ite.cond)
                    )
                } else {
                    //default
                    format!(
                        "{}\\pcif {} \\pcthen\\\\",
                        genindentation(indentation),
                        self.expression_to_tex(&ite.cond)
                    )
                    /*
                        self.write_codeblock(&ite.then_block, indentation + 1)?;
                        if !ite.else_block.0.is_empty() {
                            format!("{}\\pcelse\\\\", genindentation(indentation))?;
                            self.write_codeblock(&ite.else_block, indentation + 1)?;
                    }
                         */
                }
            }
            Statement::For(var, from, to, code, _) => {
                println!("{var:?}");
                if let Identifier::PackageIdentifier(PackageIdentifier::CodeLoopVar(
                    PackageOracleCodeLoopVarIdentifier {
                        start_comp,
                        end_comp,
                        ..
                    },
                )) = var
                {
                    format!(
                        "{}\\pcfor {} {} {} {} {} \\pcdo\\\\",
                        genindentation(indentation),
                        self.expression_to_tex(from),
                        self.forcomp_to_tex(start_comp),
                        self.ident_to_tex(var),
                        self.forcomp_to_tex(end_comp),
                        self.expression_to_tex(to)
                    )
                    //self.write_codeblock(code, indentation + 1)?;
                } else {
                    unreachable!();
                }
            }
            Statement::Sample(ident, None, maybecnt, tipe, _) => {
                let cnt = maybecnt.expect("Expected samplified input");

                format!(
                    "{}{} \\stackrel{{{}}}{{\\sample}} {}\\\\",
                    genindentation(indentation),
                    self.ident_to_tex(ident),
                    cnt,
                    self.type_to_tex(tipe)
                )
            }
            Statement::Sample(ident, Some(idxexpr), maybecnt, tipe, _) => {
                let cnt = maybecnt.expect("Expected samplified input");

                format!(
                    "{}{}[{}] \\stackrel{{{}}}{{\\samples}} {}\\\\",
                    genindentation(indentation),
                    self.ident_to_tex(ident),
                    self.expression_to_tex(idxexpr),
                    cnt,
                    self.type_to_tex(tipe)
                )
            }
            Statement::InvokeOracle(InvokeOracleStatement {
                id: ident,
                opt_idx: None,
                name,
                args,
                target_inst_name: Some(target_inst_name),
                tipe: _,
                ..
            }) => {
                format!(
                         "{}{} \\stackrel{{\\mathsf{{\\tiny{{invoke}}}}}}{{\\gets}} \\O{{{}}}({}) \\pccomment{{Pkg: {}}} \\\\",
                         genindentation(indentation),
                         self.ident_to_tex(ident), name.replace("_", "\\_"),
                         args.iter().map(|expr| self.expression_to_tex(expr)).collect::<Vec<_>>().join(", "),
                         target_inst_name.replace('_',"\\_")
                )
            }
            Statement::InvokeOracle(InvokeOracleStatement {
                id: ident,
                opt_idx: Some(idxexpr),
                name,
                args,
                target_inst_name: Some(target_inst_name),
                tipe: _,
                ..
            }) => {
                format!(
                    "{}{}[{}] \\stackrel{{\\mathsf{{\\tiny invoke}}}}{{\\gets}} \\O{{{}}}({}) \\pccomment{{Pkg: {}}} \\\\",
                         genindentation(indentation),
                         self.ident_to_tex(ident),
                         self.expression_to_tex(idxexpr),
                         name.replace("_", "\\_"),
                         args.iter().map(|expr| self.expression_to_tex(expr)).collect::<Vec<_>>().join(", "),
                         target_inst_name.replace('_',"\\_")
                )
            }
            Statement::InvokeOracle(InvokeOracleStatement {
                target_inst_name: None,
                ..
            }) => {
                unreachable!("Expect oracle-lowlevelified input")
            }
        }
    }

    pub(crate) fn write_codeblock(
        &mut self,
        codeblock: &CodeBlock,
        indentation: u8,
    ) -> std::io::Result<()> {
        for stmt in &codeblock.0 {
            writeln!(self.file, "{}", self.write_statement(stmt, indentation))?;
            
            if let Statement::IfThenElse(ite) = stmt {
                self.write_codeblock(&ite.then_block, indentation + 1)?;
                if !ite.else_block.0.is_empty() {
                    writeln!(self.file, "{}\\pcelse\\\\", genindentation(indentation))?;
                    self.write_codeblock(&ite.else_block, indentation + 1)?;
                }
            }
            if let Statement::For(_, _, _, code, _) = stmt {
                self.write_codeblock(code, indentation + 1)?;
            }
        }
        Ok(())
    }

    pub fn write_merged_codeblocks(
        &mut self,
        codeblock: &[&CodeBlock],
        indentation: u8,
    ) -> std::io::Result<()> {
        Ok(())
    }
}
