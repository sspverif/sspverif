use super::{
    common::*,
    error::{IdentifierAlreadyDeclaredError, TypeMismatchError, UndefinedIdentifierError},
    ParseContext, Rule,
};
use crate::{
    expressions::Expression,
    identifier::{
        game_ident::GameIdentifier,
        pkg_ident::{
            PackageConstIdentifier, PackageIdentifier, PackageImportsLoopVarIdentifier,
            PackageLocalIdentifier, PackageOracleArgIdentifier, PackageOracleCodeLoopVarIdentifier,
            PackageStateIdentifier,
        },
        Identifier,
    },
    package::{OracleDef, OracleSig, Package},
    statement::{CodeBlock, FilePosition, Statement},
    types::Type,
    util::{
        resolver::Named,
        scope::{self, Declaration, OracleContext, Scope, ValidityContext},
    },
    writers::smt::{
        declare::declare_const,
        exprs::{SmtAnd, SmtAssert, SmtEq2, SmtExpr, SmtIte, SmtLt, SmtLte, SmtNot},
        sorts::SmtInt,
    },
};

use miette::{Diagnostic, NamedSource, SourceSpan};
use pest::iterators::Pair;
use thiserror::Error;

use std::collections::HashMap;
use std::convert::TryInto;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct PackageParseContext<'a> {
    pub file_name: &'a str,
    pub file_content: &'a str,

    pub pkg_name: &'a str,
    pub scope: Scope,

    pub oracles: Vec<OracleDef>,
    pub state: Vec<(String, Type, SourceSpan)>,
    pub params: Vec<(String, Type, SourceSpan)>,
    pub types: Vec<(String, SourceSpan)>,
    pub imported_oracles: HashMap<String, (OracleSig, SourceSpan)>,
}

impl<'a> ParseContext<'a> {
    fn pkg_parse_context(self, pkg_name: &'a str) -> PackageParseContext {
        let mut scope = Scope::new();
        scope.enter();

        PackageParseContext {
            file_name: self.file_name,
            file_content: self.file_content,
            pkg_name,
            scope,

            oracles: vec![],
            state: vec![],
            params: vec![],
            types: vec![],
            imported_oracles: HashMap::new(),
        }
    }
}

impl<'a> PackageParseContext<'a> {
    fn named_source(&self) -> NamedSource<String> {
        NamedSource::new(self.file_name, self.file_content.to_string())
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum ParsePackageError {
    #[error("error parsing package's import oracle block: {0}")]
    #[diagnostic(transparent)]
    ParseImportOracleSig(#[from] ParseImportOraclesError),
    #[error("error parsing oracle definition: {0}")]
    #[diagnostic(transparent)]
    ParseOracleDef(#[from] ParseOracleDefError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    IdentifierAlreadyDeclared(#[from] IdentifierAlreadyDeclaredError),
}

#[derive(Error, Debug)]
pub enum ParseOracleSigError {}

impl<'a> PackageParseContext<'a> {
    fn with_scope(self, scope: Scope) -> Self {
        Self { scope, ..self }
    }
}

pub fn handle_pkg(
    file_name: &str,
    file_content: &str,
    pkg: Pair<Rule>,
) -> Result<(String, Package), ParsePackageError> {
    let mut inner = pkg.into_inner();
    let pkg_name = inner.next().unwrap().as_str();
    let spec = inner.next().unwrap();

    let ctx = ParseContext::new(file_name, file_content).pkg_parse_context(pkg_name);

    let pkg = handle_pkg_spec(ctx, spec)?;
    Ok((pkg_name.to_owned(), pkg))
}

pub enum IdentType {
    State,
    Const,
}

pub fn handle_pkg_spec(
    mut ctx: PackageParseContext,
    pkg_spec: Pair<Rule>,
) -> Result<Package, ParsePackageError> {
    // TODO(2024-04-03): get rid of the unwraps in params, state, import_oracles
    for spec in pkg_spec.into_inner() {
        match spec.as_rule() {
            Rule::types => {
                for types_list in spec.into_inner() {
                    ctx.types.append(&mut handle_types_list(types_list))
                }
            }
            Rule::params => {
                ctx.scope.enter();
                let ast = spec.into_inner().next();
                let params_option_result: Option<Result<_, _>> =
                    ast.map(|params| handle_decl_list(&mut ctx, params, IdentType::Const));

                params_option_result.transpose()?;
            }
            Rule::state => {
                ctx.scope.enter();
                let ast = spec.into_inner().next();
                let state_option_result: Option<Result<_, _>> =
                    ast.map(|state| handle_decl_list(&mut ctx, state, IdentType::State));
                state_option_result.transpose()?;
            }
            Rule::import_oracles => {
                ctx.scope.enter();
                let mut loopvar_scope = ctx.scope.clone();

                let body_ast = spec.into_inner().next().unwrap();
                handle_import_oracles_body(&mut ctx, body_ast, &mut loopvar_scope)
                    .map_err(ParsePackageError::ParseImportOracleSig)?
            }
            Rule::oracle_def => {
                handle_oracle_def(&mut ctx, spec)?;
            }
            _ => unreachable!("unhandled ast node in package: {}", spec),
        }
    }

    Ok(Package {
        name: ctx.pkg_name.to_string(),
        oracles: ctx.oracles,
        types: ctx.types,
        params: ctx.params,
        imports: ctx
            .imported_oracles
            .iter()
            .map(|(_k, (v, loc))| (v.clone(), loc.clone()))
            .collect(),
        state: ctx.state,
        split_oracles: vec![],

        file_name: ctx.file_name.to_string(),
        file_contents: ctx.file_content.to_string(),
    })
}

pub fn handle_decl_list(
    ctx: &mut PackageParseContext,
    decl_list: Pair<Rule>,
    ident_type: IdentType,
) -> Result<(), IdentifierAlreadyDeclaredError> {
    for entry in decl_list.into_inner() {
        let span = entry.as_span();
        let mut inner = entry.into_inner();
        let name_ast = inner.next().unwrap();
        let name_span = name_ast.as_span();
        let name = name_ast.as_str();
        let tipe = handle_type(inner.next().unwrap());

        let ident: Identifier = match ident_type {
            IdentType::State => {
                ctx.state.push((
                    name.to_string(),
                    tipe.clone(),
                    (span.start()..span.end()).into(),
                ));
                PackageStateIdentifier::new(
                    name.to_string(),
                    ctx.pkg_name.to_string(),
                    tipe.clone(),
                )
                .into()
            }
            IdentType::Const => {
                ctx.params.push((
                    name.to_string(),
                    tipe.clone(),
                    (span.start()..span.end()).into(),
                ));
                PackageConstIdentifier::new(
                    name.to_string(),
                    ctx.pkg_name.to_string(),
                    tipe.clone(),
                )
                .into()
            }
        };

        ctx.scope
            .declare(name, Declaration::Identifier(ident))
            .map_err(|e| {
                let scope::Error::AlreadyDefined(existing_name, _decl) = e;
                assert_eq!(name, existing_name);

                let length = name_span.end() - name_span.start();

                IdentifierAlreadyDeclaredError {
                    at: (name_span.start(), length).into(),
                    ident_name: name.to_string(),
                    source_code: NamedSource::new(ctx.file_name, ctx.file_content.to_string()),
                }
            })?;
    }

    Ok(())
}

// TODO: identifier is optional, type needs custom type info
pub fn handle_arglist(arglist: Pair<Rule>) -> Vec<(String, Type)> {
    arglist
        .into_inner()
        .map(|arg| {
            let mut inner = arg.into_inner();
            let id = inner.next().unwrap().as_str();
            let tipe = handle_type(inner.next().unwrap());
            (id.to_string(), tipe)
        })
        .collect()
}

#[derive(Error, Debug, Diagnostic)]
pub enum ParseExpressionError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    UndefinedIdentifier(UndefinedIdentifierError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    TypeMismatch(#[from] TypeMismatchError),
}

pub fn handle_expression(
    ctx: &mut PackageParseContext,
    ast: Pair<Rule>,
    expected_type: Option<&Type>,
) -> Result<Expression, ParseExpressionError> {
    let span = ast.as_span();
    let expr = match ast.as_rule() {
        Rule::expr_add => {
            if let Some(ty_expect) = expected_type {
                if ty_expect != &Type::Integer {
                    return Err(TypeMismatchError {
                        at: (span.start()..span.end()).into(),
                        expected: ty_expect.clone(),
                        got: Type::Integer,
                        source_code: ctx.named_source(),
                    }
                    .into());
                }
            }

            let mut inner = ast.into_inner();

            let lhs_ast = inner.next().unwrap();
            let rhs_ast = inner.next().unwrap();
            let span = lhs_ast.as_span();

            let lhs = handle_expression(ctx, lhs_ast, expected_type)?;

            let ty_lhs = lhs.get_type();

            if ty_lhs != Type::Integer {
                return Err(TypeMismatchError {
                    at: (span.start()..span.end()).into(),
                    expected: Type::Integer,
                    got: ty_lhs,
                    source_code: ctx.named_source(),
                }
                .into());
            }

            let rhs = handle_expression(ctx, rhs_ast, Some(&ty_lhs))?;

            Expression::Add(Box::new(lhs), Box::new(rhs))
        }
        Rule::expr_sub => {
            let mut inner = ast.into_inner();
            let lhs = handle_expression(ctx, inner.next().unwrap(), expected_type)?;
            let rhs = handle_expression(ctx, inner.next().unwrap(), expected_type)?;
            Expression::Sub(Box::new(lhs), Box::new(rhs))
        }
        Rule::expr_mul => {
            let mut inner = ast.into_inner();
            let lhs = handle_expression(ctx, inner.next().unwrap(), expected_type)?;
            let rhs = handle_expression(ctx, inner.next().unwrap(), expected_type)?;
            Expression::Mul(Box::new(lhs), Box::new(rhs))
        }
        Rule::expr_div => {
            let mut inner = ast.into_inner();
            let lhs = handle_expression(ctx, inner.next().unwrap(), expected_type)?;
            let rhs = handle_expression(ctx, inner.next().unwrap(), expected_type)?;
            Expression::Div(Box::new(lhs), Box::new(rhs))
        }

        Rule::expr_and => Expression::And(
            ast.into_inner()
                .map(|expr| handle_expression(ctx, expr, Some(&Type::Boolean)))
                .collect::<Result<_, _>>()?,
        ),
        Rule::expr_or => Expression::Or(
            ast.into_inner()
                .map(|expr| handle_expression(ctx, expr, Some(&Type::Boolean)))
                .collect::<Result<_, _>>()?,
        ),
        Rule::expr_xor => Expression::Xor(
            ast.into_inner()
                .map(|expr| handle_expression(ctx, expr, Some(&Type::Boolean)))
                .collect::<Result<_, _>>()?,
        ),
        Rule::expr_not => {
            let mut inner = ast.into_inner();
            let content = handle_expression(ctx, inner.next().unwrap(), Some(&Type::Boolean))?;
            Expression::Not(Box::new(content))
        }

        Rule::expr_equals => Expression::Equals(
            ast.into_inner()
                .map(|expr| handle_expression(ctx, expr, None))
                .collect::<Result<_, _>>()?,
        ),
        Rule::expr_not_equals => Expression::Not(Box::new(Expression::Equals(
            ast.into_inner()
                .map(|expr| handle_expression(ctx, expr, None))
                .collect::<Result<_, _>>()?,
        ))),

        Rule::expr_none => {
            let tipe = handle_type(ast.into_inner().next().unwrap());
            Expression::None(tipe)
        }
        Rule::expr_some => {
            // make sure the expected type is a maybe.
            // if yes, extract the expected type for the inner expression
            // if not, abort with an error
            let expected_type = if let Some(ty_expect) = expected_type {
                let Type::Maybe(ty) = ty_expect else {
                    return Err(TypeMismatchError {
                        at: (span.start()..span.end()).into(),
                        expected: ty_expect.clone(),
                        // TODO: maybe actually keep parsing here to produce better diagnostic
                        // output (whole type)
                        got: Type::Maybe(Box::new(Type::Unknown)),
                        source_code: ctx.named_source(),
                    }
                    .into());
                };
                Some(*ty.clone())
            } else {
                None
            };

            let expr = handle_expression(
                ctx,
                ast.into_inner().next().unwrap(),
                expected_type.as_ref(),
            )?;
            Expression::Some(Box::new(expr))
        }
        Rule::expr_unwrap => {
            let expected_type: Option<Type> =
                expected_type.map(|ty| Type::Maybe(Box::new(ty.clone())));
            let expr = handle_expression(
                ctx,
                ast.into_inner().next().unwrap(),
                expected_type.as_ref(),
            )?;
            Expression::Unwrap(Box::new(expr))
        }

        Rule::expr_newtable => {
            let mut inner = ast.into_inner();
            let idxtipe = handle_type(inner.next().unwrap());
            let valtipe = handle_type(inner.next().unwrap());
            Expression::EmptyTable(Type::Table(Box::new(idxtipe), Box::new(valtipe)))
        }
        Rule::table_access => {
            let expr_span = ast.as_span();
            let mut inner = ast.into_inner();

            let ident_ast = inner.next().unwrap();
            let ident_span = ident_ast.as_span();
            let ident_name = ident_ast.as_str();
            let ident = handle_identifier_in_code_rhs(ident_name, &ctx.scope).unwrap();

            let Some(table_type) = ident.get_type() else {
                unreachable!("this should have been a proper identifier")
            };

            let Type::Table(idx_type, val_type) = table_type else {
                return Err(ParseExpressionError::TypeMismatch(TypeMismatchError {
                    at: (ident_span.start()..ident_span.end()).into(),
                    expected: Type::Table(Box::new(Type::Unknown), Box::new(Type::Unknown)),
                    got: table_type,
                    source_code: ctx.named_source(),
                }));
            };

            let idx_expr = handle_expression(ctx, inner.next().unwrap(), Some(&*idx_type))?;

            if let Some(expected_type) = expected_type {
                if expected_type != &Type::Maybe(val_type.clone()) {
                    return Err(ParseExpressionError::TypeMismatch(TypeMismatchError {
                        at: (expr_span.start()..expr_span.end()).into(),
                        expected: expected_type.clone(),
                        got: Type::Table(idx_type, val_type),
                        source_code: ctx.named_source(),
                    }));
                }
            }

            // TODO properly parse this identifier
            Expression::TableAccess(ident, Box::new(idx_expr))
        }

        Rule::fn_call => {
            let span = ast.as_span();
            let mut inner = ast.into_inner();
            let ident = inner.next().unwrap().as_str();
            // TODO:look up the function signature and pass argument types into
            //       the expected_type arguemnts to handle_expression below
            let args = match inner.next() {
                None => vec![],
                Some(inner_args) => inner_args
                    .into_inner()
                    .map(|expr| handle_expression(ctx, expr, None))
                    .collect::<Result<_, _>>()?,
            };
            let decl = ctx
                .scope
                .lookup(ident)
                .ok_or(ParseExpressionError::UndefinedIdentifier(
                    UndefinedIdentifierError {
                        at: (span.start()..span.end()).into(),
                        ident_name: ident.to_string(),
                        source_code: NamedSource::new(ctx.file_name, ctx.file_content.to_string()),
                    },
                ))?;
            let ident = decl.into_identifier().unwrap();
            Expression::FnCall(ident, args)
        }

        Rule::identifier => {
            let span = ast.as_span();

            let name = ast.as_str().to_string();
            let decl = ctx
                .scope
                .lookup(&name)
                .ok_or(ParseExpressionError::UndefinedIdentifier(
                    UndefinedIdentifierError {
                        at: (span.start()..span.end()).into(),
                        ident_name: name.clone(),
                        source_code: NamedSource::new(ctx.file_name, ctx.file_content.to_string()),
                    },
                ))?;

            let expect_context = ValidityContext::Package;
            let got_context = decl.validity_context();
            assert_eq!(decl.validity_context(), expect_context, "invariant does not hold! when looking up name `{name}` in scope, we got declaration {decl:?}, which is valid in context {got_context:?} but we are in context {expect_context:?}.");

            let ident = match decl {
                Declaration::Identifier(ident) => ident,
                Declaration::Oracle(_, _) => {
                    todo!("handle error, user tried assigning to an oracle")
                }
            };
            Expression::Identifier(ident)
        }

        Rule::literal_boolean => {
            let litval = ast.as_str().to_string();
            Expression::BooleanLiteral(litval)
        }
        Rule::literal_integer => {
            let litval = ast.as_str().trim().to_string();

            Expression::IntegerLiteral(litval.parse().unwrap_or_else(|_| {
                // The grammar only allows ASCII_DIGIT+ here, so we should be fine?
                // Maybe if the number is too big?
                unreachable!(
                    "error at position {:?}..{:?}: could not parse as int: {}",
                    ast.as_span().start_pos().line_col(),
                    ast.as_span().end_pos().line_col(),
                    ast.as_str(),
                )
            }))
        }
        // TODO: we can't infer the type for empty sets and lists.
        //       this means we either need separate expressions for empty ones (that have a type),
        //       or they all need a type, like None
        Rule::literal_emptyset => Expression::Set(vec![]),
        Rule::expr_list => Expression::List(
            ast.into_inner()
                .map(|expr| handle_expression(ctx, expr, None))
                .collect::<Result<_, _>>()?,
        ),
        Rule::expr_tuple => Expression::Tuple(
            ast.into_inner()
                .map(|expr| handle_expression(ctx, expr, None))
                .collect::<Result<_, _>>()?,
        ),
        Rule::expr_set => Expression::Set(
            ast.into_inner()
                .map(|expr| handle_expression(ctx, expr, None))
                .collect::<Result<_, _>>()?,
        ),
        _ => unreachable!("Unhandled expression {:#?}", ast),
    };

    if let Some(expected) = expected_type {
        let got = expr.get_type();
        let at: SourceSpan = (span.start()..span.end()).into();

        if expected != &got {
            let expected = expected.clone();

            return Err(ParseExpressionError::TypeMismatch(TypeMismatchError {
                at,
                expected,
                got,
                source_code: NamedSource::new(ctx.file_name, ctx.file_content.to_string()),
            }));
        }
    }

    Ok(expr)
}

#[derive(Error, Debug, Diagnostic)]
pub enum ParseCodeError {
    #[error("error parsing expression: {0}")]
    #[diagnostic(transparent)]
    ParseExpression(#[from] ParseExpressionError),
    #[error("error parsing identifier: {0}")]
    #[diagnostic(transparent)]
    ParseIdentifier(#[from] ParseIdentifierError),
    #[error("undefined oracle: {0}")]
    UndefinedOracle(String),
    #[error("invoking an identifier that is not an oracle: {0:?}")]
    InvokingNonOracle(Identifier),

    #[error(transparent)]
    #[diagnostic(transparent)]
    UndefinedIdentifier(#[from] UndefinedIdentifierError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    TypeMismatch(#[from] TypeMismatchError),
}

pub fn handle_identifier_in_code_rhs(
    name: &str,
    scope: &Scope,
) -> Result<Identifier, ParseIdentifierError> {
    let ident = scope
        .lookup(name)
        .ok_or(ParseIdentifierError::Undefined(name.to_string()))?
        .into_identifier()
        .unwrap_or_else(|decl| {
            panic!(
                "expected an identifier, got a declaration {decl:?}",
                decl = decl
            )
        });

    Ok(ident)
}

pub fn handle_identifier_in_code_lhs(
    ctx: &mut PackageParseContext,
    name_ast: Pair<Rule>,
    oracle_name: &str,
    expected_type: Type,
) -> Result<Identifier, ParseIdentifierError> {
    let name = name_ast.as_str();
    let span = name_ast.as_span();

    let scope = &mut ctx.scope;

    let ident = if let Some(decl) = scope.lookup(name) {
        decl.into_identifier()
            .map_err(|decl| ParseIdentifierError::InvalidLeftHandSide(name.to_string(), decl))?
    } else {
        let ident =
            Identifier::PackageIdentifier(PackageIdentifier::Local(PackageLocalIdentifier {
                pkg_name: ctx.pkg_name.to_string(),
                oracle_name: oracle_name.to_string(),
                name: name.to_string(),
                tipe: expected_type.clone(),
                pkg_inst_name: None,
                game_name: None,
                game_inst_name: None,
                proof_name: None,
            }));

        scope
            .declare(name, Declaration::Identifier(ident.clone()))
            .map_err(ParseIdentifierError::ScopeDeclareError)?;

        ident
    };

    let got = ident.get_type().unwrap();
    if got != expected_type {
        Err(ParseIdentifierError::TypeMismatch(TypeMismatchError {
            at: (span.start()..span.end()).into(),
            expected: expected_type,
            got,
            source_code: ctx.named_source(),
        }))
    } else {
        Ok(ident)
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum ParseIdentifierError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    ParseExpression(ParseExpressionError),
    #[error(transparent)]
    ScopeDeclareError(crate::util::scope::Error),
    #[error("found undefined variable `{0}`.")]
    Undefined(String),
    #[error(transparent)]
    #[diagnostic(transparent)]
    TypeMismatch(TypeMismatchError),
    #[error("error parsing left-hand-side name `{0}`: expected an identifier, got {1:?}")]
    InvalidLeftHandSide(String, Declaration),
}

pub fn handle_code(
    ctx: &mut PackageParseContext,
    code: Pair<Rule>,
    oracle_sig: &OracleSig,
) -> Result<CodeBlock, ParseCodeError> {
    let oracle_name = &oracle_sig.name;
    code.into_inner()
        .map(|stmt| {
            let span = stmt.as_span();
            let source_span = SourceSpan::from(span.start()..span.end());
            //let file_pos = FilePosition::from_span(ctx.file_name, span);

            let stmt = match stmt.as_rule() {
                // assign | return_stmt | abort | ite
                Rule::ite => {
                    let mut inner = stmt.into_inner();
                    let cond_expr = handle_expression(ctx, inner.next().unwrap(), Some(&Type::Boolean))?;
                    let ifcode = handle_code(
                        ctx,
                        inner.next().unwrap(),
                        oracle_sig,
                    )?;
                    let maybe_elsecode = inner.next();
                    let elsecode = match maybe_elsecode {
                        None => CodeBlock(vec![]),
                        Some(c) => handle_code(ctx, c, oracle_sig)?,
                    };

                    Statement::IfThenElse(cond_expr, ifcode, elsecode, source_span)
                }
                Rule::return_stmt => {
                    let mut inner = stmt.into_inner();
                    let maybe_expr = inner.next();


                    let expr = maybe_expr.map(|expr| handle_expression(ctx, expr, Some(&oracle_sig.tipe))).transpose()?;
                    Statement::Return(expr, source_span)
                }
                Rule::assert => {
                    let mut inner = stmt.into_inner();
                    let expr = handle_expression(ctx, inner.next().unwrap(), Some(&Type::Boolean))?;
                    Statement::IfThenElse(
                        expr,
                        CodeBlock(vec![]),
                        CodeBlock(vec![Statement::Abort(source_span.clone())]),
                         source_span,
                    )
                }
                Rule::abort => Statement::Abort(source_span),
                Rule::sample => {
                    let mut inner = stmt.into_inner();
                    let name_ast = inner.next().unwrap();
                    let tipe = handle_type(inner.next().unwrap());
                    let ident = handle_identifier_in_code_lhs(
                        ctx,
                        name_ast,
                        oracle_name,
                        tipe.clone(),
                    )
                    .map_err(ParseCodeError::ParseIdentifier)?;
                    Statement::Sample(ident, None, None, tipe, source_span)
                }

                Rule::assign => {
                    let mut inner = stmt.into_inner();
                    let name_ast = inner.next().unwrap();
                    // it's fine to use a None expected_type here, because we later check that the
                    // resulting type matches the identifier
                    // maybe we could produce better errors (or have better type inference?) if we
                    // first checked whether the identifier exists, and if yes use that as the
                    // expected type here?
                    let expr = handle_expression(ctx, inner.next().unwrap(), None)
                        .map_err(ParseCodeError::ParseExpression)?;

                    let expected_type = expr.get_type();
                    let ident = handle_identifier_in_code_lhs(
                        ctx,
                        name_ast,
                        oracle_name,
                        expected_type,
                    )
                    .map_err(ParseCodeError::ParseIdentifier)?;

                    Statement::Assign(ident, None, expr, source_span)
                }

                Rule::table_sample => {
                    let mut inner = stmt.into_inner();
                    let name_ast = inner.next().unwrap();
                    let index = handle_expression(ctx, inner.next().unwrap(), None)?;
                    let tipe = handle_type(inner.next().unwrap());
                    let ident = handle_identifier_in_code_lhs(
                        ctx,
                        name_ast,
                        oracle_name,
                        Type::Table(Box::new(index.get_type()),  Box::new(tipe.clone())),
                    )
                    .map_err(ParseCodeError::ParseIdentifier)?;
                    Statement::Sample(ident, Some(index), None, tipe, source_span)
                }

                Rule::table_assign => {
                    let mut inner = stmt.into_inner();

                    let name_ast = inner.next().unwrap();
                    let Some(Declaration::Identifier(ident)) = ctx.scope.lookup(name_ast.as_str()) else {
                        let span = name_ast.as_span();
                        return Err(UndefinedIdentifierError{
                            source_code: ctx.named_source(),
                            at: (span.start()..span.end()).into(),
                            ident_name: name_ast.as_str().to_string(),
                        }.into());
                    };

                    let ty_ident = ident.get_type().unwrap();
                    let Type::Table(ty_key, ty_value_without_maybe) = &ty_ident else {
                        let span = name_ast.as_span();
                        return Err(TypeMismatchError{
                            at: (span.start()..span.end()).into(),
                            expected: Type::Table(Box::new(Type::Unknown), Box::new(Type::Unknown)),
                            got: ty_ident,
                            source_code: ctx.named_source(),
                        }.into())
                    };

                    let ty_value_with_maybe = Type::Maybe(ty_value_without_maybe.clone());

                    let index = handle_expression(ctx, inner.next().unwrap(), Some(&**ty_key))?;
                    let expr = handle_expression(ctx, inner.next().unwrap(), Some(&ty_value_with_maybe))?;

                    let expected_type = match expr.get_type() {
                        Type::Maybe(t) => Type::Table(Box::new(index.get_type()), t),
                        _ => panic!("assigning non-maybe value to table {expr:?} in {oracle_name}, {pkg_name}", expr = expr, oracle_name=oracle_name, pkg_name=ctx.pkg_name),
                    };

                    let ident = handle_identifier_in_code_lhs(
                        ctx,
                        name_ast,
                        oracle_name,
                        expected_type,
                    )
                    .map_err(ParseCodeError::ParseIdentifier)?;

                    Statement::Assign(ident, Some(index), expr, source_span)
                }

                Rule::invocation => {
                    let mut inner = stmt.into_inner();
                    let target_ident_name_ast = inner.next().unwrap();
                    let maybe_index = inner.next().unwrap();
                    // TODO: this should be used in type checking somehow
                    let (opt_idx, oracle_inv) = if maybe_index.as_rule() == Rule::table_index {
                        let mut inner_index = maybe_index.into_inner();
                        let index =
                            handle_expression(ctx, inner_index.next().unwrap(), None)?;
                        (Some(index), inner.next().unwrap())
                    } else {
                        (None, maybe_index)
                    };

                    assert!(matches!(oracle_inv.as_rule(), Rule::oracle_call));

                    let mut inner = oracle_inv.into_inner();
                    let oracle_name = inner.next().unwrap().as_str();

                    let mut args = vec![];
                    let mut dst_inst_index = None;

                    for ast in inner {
                        match ast.as_rule() {
                            Rule::oracle_call_index => {
                                let index_expr_ast = ast.into_inner().next().unwrap();
                                dst_inst_index =
                                    Some(handle_expression(ctx, index_expr_ast, Some(&Type::Integer))?);
                            }
                            Rule::fn_call_arglist => {
                                // TODO: figure out the types of the arguments and provide them to
                                // the parser
                                let arglist: Result<Vec<_>, _> = ast
                                    .into_inner()
                                    .map(|expr| handle_expression(ctx, expr, None))
                                    .collect();
                                let arglist = arglist?;
                                args.extend(arglist.into_iter())
                            }
                            _ => unreachable!(),
                        }
                    }

                    let oracle_decl = ctx.scope
                        .lookup(oracle_name)
                        .ok_or(ParseCodeError::UndefinedOracle(oracle_name.to_string()))?;

                    let expected_type = match (oracle_decl, opt_idx.clone()) {
                        (Declaration::Oracle(_context, oracle_sig), None) => Ok(oracle_sig.tipe),
                        (Declaration::Oracle(_context, oracle_sig), Some(idx)) => Ok(Type::Table(
                            Box::new(idx.get_type()),
                            Box::new(oracle_sig.tipe)
                        )),
                        (Declaration::Identifier(ident), _) => {
                            Err(ParseCodeError::InvokingNonOracle(ident))
                        }
                    }?;

                    let target_ident = handle_identifier_in_code_lhs(
                        ctx,
                        target_ident_name_ast,
                        oracle_name,
                        expected_type,
                    )
                    .map_err(ParseCodeError::ParseIdentifier)?;

                    Statement::InvokeOracle {
                        id: target_ident,
                        opt_idx,
                        opt_dst_inst_idx: dst_inst_index,
                        name: oracle_name.to_owned(),
                        args,
                        target_inst_name: None,
                        tipe: None,
                        file_pos: source_span,
                    }
                }
                Rule::parse => {
                    let mut inner = stmt.into_inner();
                    let list = inner.next().unwrap();
                    let expr = inner.next().unwrap();

                    let expr = handle_expression(ctx,expr, None)?;
                    let tipe = expr.get_type();

                    let tipes = match tipe {
                        Type::Tuple(tipes) => tipes,
                        other => panic!("expected tuple type in parse, but got {other:?} in {file_name}, {pkg_name}, {oracle_name}, {expr:?}", other=other, file_name=ctx.file_name, pkg_name=ctx.pkg_name, oracle_name=oracle_name, expr=expr)
                    };

                    let idents = list
                        .into_inner()
                        .enumerate()
                        .map(|(i, ident_name)| {
                            handle_identifier_in_code_lhs(ctx, ident_name,  oracle_name, tipes[i].clone())
                                .map_err(ParseCodeError::ParseIdentifier)
                        })
                        .collect::<Result<_,_>>()?;


                    Statement::Parse(idents, expr, source_span)
                }
                Rule::for_ => {
                    let mut parsed: Vec<Pair<Rule>> = stmt.into_inner().collect();
                    let decl_var_name = parsed[0].as_str();
                    let lower_bound = handle_expression(ctx, parsed.remove(1), Some(&Type::Integer))?;
                    let lower_bound_type = parsed[1].as_str();
                    let bound_var_name = parsed[2].as_str();
                    let upper_bound_type = parsed[3].as_str();
                    let upper_bound = handle_expression(ctx, parsed.remove(4), Some(&Type::Integer))?;

                    if decl_var_name != bound_var_name {
                        todo!("return proper error here")
                    }

                    let lower_bound_type = match lower_bound_type {
                        "<" => ForComp::Lt,
                        "<=" => ForComp::Lte,
                        _ => panic!(),
                    };

                    let upper_bound_type = match upper_bound_type {
                        "<" => ForComp::Lt,
                        "<=" => ForComp::Lte,
                        _ => panic!(),
                    };
                    let loopvar = PackageOracleCodeLoopVarIdentifier {
                        name: decl_var_name.to_string(),
                        pkg_name: ctx.pkg_name.to_string(),
                        start: Box::new(lower_bound.clone()),
                        end: Box::new(upper_bound.clone()),
                        start_comp: lower_bound_type,
                        end_comp: upper_bound_type,
                        pkg_inst_name: None,
                        game_name: None,
                        game_inst_name: None,
                        proof_name: None,
                    };
                    let loopvar: Identifier = loopvar.into();

                    ctx.scope.enter();
                    ctx.scope
                        .declare(decl_var_name, Declaration::Identifier(loopvar.clone()))
                        .unwrap();

                    let body =
                        handle_code(ctx, parsed.remove(4),oracle_sig)?;
                    ctx.scope.leave();

                    Statement::For(loopvar, lower_bound, upper_bound, body, source_span)
                }
                _ => {
                    unreachable!("{:#?}", stmt)
                }
            };

            Ok(stmt)
        })
        .collect::<Result<_, _>>()
        .map(CodeBlock)
}

#[derive(Error, Debug, Diagnostic)]
pub enum ParseOracleDefError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    ParseCode(#[from] ParseCodeError),
}

pub fn handle_oracle_def(
    ctx: &mut PackageParseContext,
    oracle_def: Pair<Rule>,
) -> Result<(), ParseOracleDefError> {
    let span = oracle_def.as_span();
    let source_span = SourceSpan::from(span.start()..span.end());
    let mut inner = oracle_def.into_inner();

    let sig = handle_oracle_sig(inner.next().unwrap()).unwrap();

    ctx.scope.enter();

    for (name, tipe) in &sig.args {
        ctx.scope
            .declare(
                name,
                Declaration::Identifier(Identifier::PackageIdentifier(
                    PackageIdentifier::OracleArg(PackageOracleArgIdentifier {
                        pkg_name: ctx.pkg_name.to_string(),
                        oracle_name: sig.name.clone(),
                        name: name.clone(),
                        tipe: tipe.clone(),
                        pkg_inst_name: None,
                        game_name: None,
                        game_inst_name: None,
                        proof_name: None,
                    }),
                )),
            )
            .unwrap();
    }

    let code =
        handle_code(ctx, inner.next().unwrap(), &sig).map_err(ParseOracleDefError::ParseCode)?;

    ctx.scope.leave();

    let oracle_def = OracleDef {
        sig,
        code,
        file_pos: source_span,
    };

    ctx.oracles.push(oracle_def);

    Ok(())
}

pub fn handle_oracle_sig(oracle_sig: Pair<Rule>) -> Result<OracleSig, ParseOracleSigError> {
    let mut inner = oracle_sig.into_inner();
    let name = inner.next().unwrap().as_str();
    let maybe_arglist = inner.next().unwrap();
    let args = if maybe_arglist.as_str() == "" {
        vec![]
    } else {
        handle_arglist(maybe_arglist.into_inner().next().unwrap())
    };

    let maybe_tipe = inner.next();
    let tipe = match maybe_tipe {
        None => Type::Empty,
        Some(t) => handle_type(t),
    };

    Ok(OracleSig {
        name: name.to_string(),
        tipe,
        args,
        multi_inst_idx: MultiInstanceIndices::new(vec![]),
    })
}

#[derive(Debug)]
pub enum ParseImportsOracleSigError {
    IndexParseError(ParseExpressionError),
}

impl std::fmt::Display for ParseImportsOracleSigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseImportsOracleSigError::IndexParseError(e) => write!(f, "error parsing index: {e}"),
        }
    }
}

impl std::error::Error for ParseImportsOracleSigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseImportsOracleSigError::IndexParseError(e) => Some(e),
        }
    }
}

pub fn handle_oracle_imports_oracle_sig(
    ctx: &mut PackageParseContext,
    oracle_sig: Pair<Rule>,
    loopvar_scope: &Scope,
) -> Result<OracleSig, ParseImportsOracleSigError> {
    println!("{:?}", oracle_sig.as_rule());

    let _span = oracle_sig.as_span();

    let mut inner = oracle_sig.into_inner();
    let name = inner.next().unwrap().as_str();

    let (multi_inst_idx, args) = {
        let mut multi_inst_idx = vec![];
        let mut arglist = vec![];

        while let Some(next) = inner.peek() {
            match next.as_rule() {
                Rule::indices_expr => {
                    let mut loopvar_ctx = ctx.clone().with_scope(loopvar_scope.clone());
                    let indices: Vec<_> = next
                        .into_inner()
                        .map(|expr| handle_expression(&mut loopvar_ctx, expr, Some(&Type::Integer)))
                        .collect::<Result<_, _>>()
                        .map_err(ParseImportsOracleSigError::IndexParseError)?;
                    multi_inst_idx.extend_from_slice(&indices);
                    inner.next();
                }
                Rule::fn_maybe_arglist => {
                    if !next.as_str().is_empty() {
                        arglist = handle_arglist(next.into_inner().next().unwrap());
                    }
                    inner.next();
                }
                _ => break,
            }
        }

        (MultiInstanceIndices::new(multi_inst_idx), arglist)
    };

    let maybe_tipe = inner.next();
    let tipe = match maybe_tipe {
        None => Type::Empty,
        Some(t) => handle_type(t),
    };

    Ok(OracleSig {
        name: name.to_string(),
        tipe,
        args,
        multi_inst_idx,
    })
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ForComp {
    Lt,
    Lte,
}

#[derive(Debug)]
pub struct ForCompError(String);

impl std::fmt::Display for ForCompError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "not a valid comparison operator for for loops: {:?}",
            self.0
        )
    }
}

impl std::error::Error for ForCompError {}

impl crate::error::LocationError for ParseImportOraclesError {
    fn file_pos(&self) -> &FilePosition {
        match self {
            ParseImportOraclesError::IdentifierMismatch(_, _, file_pos)
            | ParseImportOraclesError::ParseImportOracleSig(_, file_pos)
            | ParseImportOraclesError::ParseStartExpression(_, file_pos)
            | ParseImportOraclesError::ParseEndExpression(_, file_pos)
            | ParseImportOraclesError::InvalidStartComparison(_, file_pos)
            | ParseImportOraclesError::InvalidEndComparison(_, file_pos) => file_pos,
            ParseImportOraclesError::DeclareError(_, file_pos) => file_pos,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MultiInstanceIndices {
    pub(crate) indices: Vec<Expression>,
}
impl MultiInstanceIndices {
    pub(crate) fn new(indices: Vec<Expression>) -> Self {
        Self { indices }
    }

    pub(crate) fn empty() -> Self {
        Self { indices: vec![] }
    }
}

/// A [`MultiInstanceIndicesGroup`] contains a list of [`MultiInstanceIndices`] that all represent
/// the same index, but cover different ranges. The purpose is that the group folds the individual
/// elements into a single one, by merging the ranges specified in the [`ForSpec`] entries.
pub struct MultiInstanceIndicesGroup(Vec<MultiInstanceIndices>);

impl MultiInstanceIndicesGroup {
    pub fn new(v: Vec<MultiInstanceIndices>) -> Self {
        Self(v)
    }

    pub(crate) fn smt_check_total(
        &self,
        assumptions: Vec<SmtExpr>,
        consts: &[&str],
        varname: &str,
    ) -> Vec<SmtExpr> {
        let declares: Vec<_> = consts
            .iter()
            .map(|const_name| declare_const(*const_name, SmtInt))
            .collect();

        let predicate = self.smt_totality_check_function(varname);

        let neg_claim = SmtNot(SmtEq2 {
            lhs: predicate,
            rhs: 1usize,
        })
        .into();

        let and_terms = assumptions.into_iter().chain(vec![neg_claim]).collect();

        let assert = SmtAssert(SmtAnd(and_terms));

        let mut out_statements = declares;
        out_statements.push(assert.into());

        out_statements
        /*
         *
         * declare const n Int
         * ...
         * declare const x Int
         *
         * assert AND(assumptions)  => sum of predicates = 1
         * -> expect unsat
         *
         */
    }

    fn smt_totality_check_function(&self, varname: &str) -> SmtExpr {
        let terms = self.0.iter().map(|indices| {
            SmtIte {
                cond: indices.smt_range_predicate(varname),
                then: 1,
                els: 0,
            }
            .into()
        });

        let add = ["+"].iter().map(|&add| add.into());
        let zero = [0].iter().map(|&zero| zero.into());

        // add the zero term so the operation doesn't fail if there are no terms
        SmtExpr::List(add.chain(terms).chain(zero).collect())
    }
}

impl MultiInstanceIndices {
    /// returns smt code that checks whether a variable with name `varname` is in the range.
    /// currently only works for one-dimensional indices and panics for higher dimensions.
    pub(crate) fn smt_range_predicate(&self, varname: &str) -> SmtExpr {
        //assert!(self.indices.len() == 1);
        match &self.indices[0] {
            Expression::IntegerLiteral(index) => SmtEq2 {
                lhs: *index,
                rhs: varname,
            }
            .into(),
            // I don't think we need to check totality for imports inside the package's import
            // block, so we don't need to handle ImportsLoopVar.
            // Expression::Identifier(Identifier::PackageIdentifier(
            // PackageIdentifier::ImportsLoopVar(loopvar),
            // )) => {
            // let start_comp: SmtExpr = match loopvar.start_comp {
            // ForComp::Lt => SmtLt((*loopvar.start).clone(), varname).into(),
            // ForComp::Lte => SmtLte((*loopvar.start).clone(), varname).into(),
            // };
            //
            // let end_comp: SmtExpr = match loopvar.end_comp {
            // ForComp::Lt => SmtLt(varname, (*loopvar.end).clone()).into(),
            // ForComp::Lte => SmtLte(varname, (*loopvar.end).clone()).into(),
            // };
            //
            // SmtAnd(vec![start_comp, end_comp]).into()
            // }
            Expression::Identifier(Identifier::GameIdentifier(GameIdentifier::Const(
                game_const_ident,
            ))) => SmtEq2 {
                lhs: &game_const_ident.name,
                rhs: varname,
            }
            .into(),
            Expression::Identifier(Identifier::GameIdentifier(GameIdentifier::LoopVar(
                game_loop_var,
            ))) => {
                let lower_comp: SmtExpr = match game_loop_var.start_comp {
                    ForComp::Lt => SmtLt((*game_loop_var.start).clone(), varname).into(),
                    ForComp::Lte => SmtLte((*game_loop_var.start).clone(), varname).into(),
                };

                let upper_comp: SmtExpr = match game_loop_var.end_comp {
                    ForComp::Lt => SmtLt((*game_loop_var.end).clone(), varname).into(),
                    ForComp::Lte => SmtLte((*game_loop_var.end).clone(), varname).into(),
                };

                SmtAnd(vec![lower_comp, upper_comp]).into()
            }
            Expression::Identifier(Identifier::Parameter(pkg_const)) => SmtEq2 {
                lhs: pkg_const.name_in_comp.clone(),
                rhs: varname,
            }
            .into(),
            other => unreachable!(
                "in smt_range_predicate, found unhandled expression variant {expr:?}",
                expr = other
            ),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct ForSpec {
    ident: Identifier,
    start: Expression,
    end: Expression,
    start_comp: ForComp,
    end_comp: ForComp,
}

impl Named for ForSpec {
    fn as_name(&self) -> &str {
        self.ident.ident_ref()
    }
}

impl ForSpec {
    pub fn new(
        ident: Identifier,
        start: Expression,
        end: Expression,
        start_comp: ForComp,
        end_comp: ForComp,
    ) -> Self {
        Self {
            ident,
            start,
            end,
            start_comp,
            end_comp,
        }
    }

    pub fn ident(&self) -> &Identifier {
        &self.ident
    }

    pub fn start(&self) -> &Expression {
        &self.start
    }

    pub fn end(&self) -> &Expression {
        &self.end
    }

    pub fn start_comp(&self) -> &ForComp {
        &self.start_comp
    }

    pub fn end_comp(&self) -> &ForComp {
        &self.end_comp
    }

    pub(crate) fn map_identifiers<F: Fn(&Identifier) -> Identifier>(&self, f: F) -> Self {
        Self {
            start: map_ident_expr(&self.start, &f),
            end: map_ident_expr(&self.end, &f),
            ..self.clone()
        }
    }
}

fn map_ident_expr<F: Fn(&Identifier) -> Identifier>(expr: &Expression, f: &F) -> Expression {
    if let Expression::Identifier(id) = expr {
        Expression::Identifier(f(id))
    } else {
        expr.clone()
    }
}

impl std::convert::TryFrom<&str> for ForComp {
    type Error = ForCompError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "<=" => Ok(ForComp::Lte),
            "<" => Ok(ForComp::Lt),
            _ => Err(ForCompError(value.to_string())),
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum ParseImportOraclesError {
    #[error("for loop spec uses different identifiers: {0:?} != {1:?}")]
    IdentifierMismatch(String, String, FilePosition),
    #[error("error parsing loop start expression: {0}")]
    ParseStartExpression(ParseExpressionError, FilePosition),
    #[error("error parsing loop end expression: {0}")]
    ParseEndExpression(ParseExpressionError, FilePosition),
    #[error("first loop comparison is invalid: {0}")]
    InvalidStartComparison(ForCompError, FilePosition),
    #[error("second loop comparison is invalid: {0}")]
    InvalidEndComparison(ForCompError, FilePosition),
    #[error("error parsing import oracle signature: {0}")]
    ParseImportOracleSig(ParseImportsOracleSigError, FilePosition),
    #[error("erroring declaring identifier: {0}")]
    DeclareError(crate::util::scope::Error, FilePosition),
}

pub fn handle_import_oracles_body(
    ctx: &mut PackageParseContext,
    ast: Pair<Rule>,
    loopvar_scope: &mut Scope,
) -> Result<(), ParseImportOraclesError> {
    let pkg_name = ctx.pkg_name;
    assert_eq!(ast.as_rule(), Rule::import_oracles_body);

    for entry in ast.into_inner() {
        match entry.as_rule() {
            Rule::import_oracles_oracle_sig => {
                let span = entry.as_span();
                let source_span = SourceSpan::from(span.start()..span.end());
                let file_pos = FilePosition::from_span(ctx.file_name, entry.as_span());
                let sig =
                    handle_oracle_imports_oracle_sig(ctx, entry, loopvar_scope).map_err(|e| {
                        ParseImportOraclesError::ParseImportOracleSig(e, file_pos.clone())
                    })?;
                ctx.imported_oracles
                    .insert(sig.name.clone(), (sig.clone(), source_span));
                ctx.scope
                    .declare(
                        &sig.name,
                        Declaration::Oracle(
                            OracleContext::Package {
                                pkg_name: pkg_name.to_string(),
                            },
                            sig.clone(),
                        ),
                    )
                    .map_err(|err| ParseImportOraclesError::DeclareError(err, file_pos))?;
            }

            Rule::import_oracles_for => {
                let for_span = entry.as_span();
                let mut for_ast = entry.into_inner();

                let ident_ast = for_ast.next().unwrap();
                let ident = ident_ast.as_str().to_string();

                let for_start_ast = for_ast.next().unwrap();
                let for_start_file_pos =
                    FilePosition::from_span(ctx.file_name, for_start_ast.as_span());
                let for_start = handle_expression(ctx, for_start_ast, Some(&Type::Integer))
                    .map_err(|e| {
                        ParseImportOraclesError::ParseStartExpression(e, for_start_file_pos)
                    })?;

                let start_comp_ast = for_ast.next().unwrap();
                let start_comp_filepos =
                    FilePosition::from_span(ctx.file_name, start_comp_ast.as_span());
                let start_comp: ForComp = start_comp_ast.as_str().try_into().map_err(|e| {
                    ParseImportOraclesError::InvalidStartComparison(e, start_comp_filepos)
                })?;

                let ident2_ast = for_ast.next().unwrap();
                let ident2_span = ident2_ast.as_span();
                let ident2 = ident2_ast.as_str().to_string();

                let end_comp_ast = for_ast.next().unwrap();
                let end_comp_filepos =
                    FilePosition::from_span(ctx.file_name, end_comp_ast.as_span());
                let end_comp: ForComp = end_comp_ast.as_str().try_into().map_err(|e| {
                    ParseImportOraclesError::InvalidEndComparison(e, end_comp_filepos)
                })?;

                let for_end_ast = for_ast.next().unwrap();
                let for_end_file_pos =
                    FilePosition::from_span(ctx.file_name, for_end_ast.as_span());
                let for_end =
                    handle_expression(ctx, for_end_ast, Some(&Type::Integer)).map_err(|e| {
                        ParseImportOraclesError::ParseEndExpression(e, for_end_file_pos)
                    })?;

                if ident != ident2 {
                    return Err(ParseImportOraclesError::IdentifierMismatch(
                        ident,
                        ident2,
                        FilePosition::from_span(ctx.file_name, ident2_span),
                    ));
                }

                let ident_data = PackageImportsLoopVarIdentifier {
                    pkg_name: pkg_name.to_string(),
                    name: ident.clone(),
                    start: Box::new(for_start.clone()),
                    end: Box::new(for_end.clone()),
                    start_comp,
                    end_comp,
                    pkg_inst_name: None,
                    game_name: None,
                    game_inst_name: None,
                    proof_name: None,
                };

                let identifier =
                    Identifier::PackageIdentifier(PackageIdentifier::ImportsLoopVar(ident_data));

                loopvar_scope.enter();

                loopvar_scope
                    .declare(&ident, Declaration::Identifier(identifier))
                    .map_err(|e| {
                        ParseImportOraclesError::DeclareError(
                            e,
                            FilePosition::from_span(ctx.file_name, for_span),
                        )
                    })?;

                handle_import_oracles_body(ctx, for_ast.next().unwrap(), loopvar_scope)?;
                loopvar_scope.leave();
            }

            _ => unreachable!(),
        }
    }
    Ok(())
}

pub fn handle_types_list(types: Pair<Rule>) -> Vec<(String, SourceSpan)> {
    types
        .into_inner()
        .map(|entry| {
            let span = entry.as_span();
            (
                entry.as_str().to_string(),
                (span.start()..span.end()).into(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::parser::{
        package::{handle_pkg, ParseExpressionError},
        SspParser,
    };

    fn unwrap_parse_err<T>(res: Result<T, pest::error::Error<crate::parser::Rule>>) -> T {
        match res {
            Ok(v) => v,
            Err(err) => panic!("parse error: {err}", err = err),
        }
    }

    fn fail_parse_pkg(code: &str, name: &str) -> ParsePackageError {
        let mut pkg_pairs = unwrap_parse_err(SspParser::parse_package(code));
        handle_pkg(name, code, pkg_pairs.next().unwrap())
            .expect_err("expected to fail parsing the package")
    }

    const TINY_BAD_PKG_1_CODE: &str = r#"package TinyBadPkg1 {
            params {
              n: Integer,
            }

            oracle N() -> String {
              return n;
            }
        }"#;

    const TINY_BAD_PKG_2_CODE: &str = r#"package TinyBadPkg2 {
            oracle N() -> String {
              return n;
            }
        }"#;

    const TINY_BAD_PKG_3_CODE: &str = r#"package TinyBadPkg3 {
            oracle N() -> Integer {
              return (true + false);
            }
        }"#;

    const TINY_BAD_PKG_4_CODE: &str = r#"package TinyBadPkg4 {
            oracle N() -> Integer {
              return (true + 3);
            }
        }"#;

    const TINY_BAD_PKG_5_CODE: &str = r#"package TinyBadPkg5 {
            oracle N() -> Integer {
              return (3 + true);
            }
        }"#;

    const TINY_BAD_PKG_6_CODE: &str = r#"package TinyBadPkg6 {
            oracle N() -> Bool {
              return (3 + 2);
            }
        }"#;

    #[test]
    fn wrong_return_type_fails() {
        let err = fail_parse_pkg(TINY_BAD_PKG_1_CODE, "tiny-bad-pkg-1");

        assert!(matches!(
            err,
            ParsePackageError::ParseOracleDef(ParseOracleDefError::ParseCode(
                ParseCodeError::ParseExpression(ParseExpressionError::TypeMismatch(
                    TypeMismatchError {
                        expected: Type::String,
                        got: Type::Integer,
                        ..
                    }
                ))
            ))
        ))
    }

    #[test]
    fn missing_identifier_fails() {
        let err = fail_parse_pkg(TINY_BAD_PKG_2_CODE, "tiny-bad-pkg-2");

        assert!(matches!(
            err,
            ParsePackageError::ParseOracleDef(ParseOracleDefError::ParseCode(
                ParseCodeError::ParseExpression(ParseExpressionError::UndefinedIdentifier(
                    UndefinedIdentifierError { ref ident_name, .. }
                ))
            )) if ident_name.as_str() == "n"

        ));

        println!("{:?}", miette::Report::new(err));
    }

    #[test]
    fn bad_add_fails_1() {
        let err = fail_parse_pkg(TINY_BAD_PKG_3_CODE, "tiny-bad-pkg-3");

        assert!(
            matches!(
                err,
                ParsePackageError::ParseOracleDef(ParseOracleDefError::ParseCode(
                    ParseCodeError::ParseExpression(ParseExpressionError::TypeMismatch(
                        TypeMismatchError {
                            expected: Type::Integer,
                            got: Type::Boolean,
                            ref at,
                            ref source_code,
                        }
                    ))
                )) if slice_source_span(source_code, at) == "true"
            ),
            "got: {:#?}",
            err
        );

        println!("{:?}", miette::Report::new(err));
    }

    #[test]
    fn bad_add_fails_2() {
        let err = fail_parse_pkg(TINY_BAD_PKG_4_CODE, "tiny-bad-pkg-4");

        assert!(
            matches!(
                err,
                ParsePackageError::ParseOracleDef(ParseOracleDefError::ParseCode(
                    ParseCodeError::ParseExpression(ParseExpressionError::TypeMismatch(
                        TypeMismatchError {
                            expected: Type::Integer,
                            got: Type::Boolean,
                            ref at,
                            ref source_code,
                        }
                    ))
                )) if slice_source_span(source_code, at) == "true"
            ),
            "got: {:#?}",
            err
        );

        println!("{:?}", miette::Report::new(err));
    }

    #[test]
    fn bad_add_fails_3() {
        let err = fail_parse_pkg(TINY_BAD_PKG_5_CODE, "tiny-bad-pkg-5");

        assert!(
            matches!(
                err,
                ParsePackageError::ParseOracleDef(ParseOracleDefError::ParseCode(
                    ParseCodeError::ParseExpression(ParseExpressionError::TypeMismatch(
                        TypeMismatchError {
                            expected: Type::Integer,
                            got: Type::Boolean,
                            ref at,
                            ref source_code,
                        }
                    ))
                ))
                    if slice_source_span(source_code, at) == "true"
            ),
            "got: {:#?}",
            err
        );

        println!("{:?}", miette::Report::new(err));
    }
    #[test]
    fn bad_add_fails_4() {
        let err = fail_parse_pkg(TINY_BAD_PKG_6_CODE, "tiny-bad-pkg-6");

        assert!(
            matches!(
                err,
                ParsePackageError::ParseOracleDef(ParseOracleDefError::ParseCode(
                    ParseCodeError::ParseExpression(ParseExpressionError::TypeMismatch(
                        TypeMismatchError {
                            expected: Type::Boolean,
                            got: Type::Integer,
                            ref at,
                            ref source_code,
                        },
                    )),
                ))
                    if slice_source_span(source_code, at) == "(3 + 2)"
            ),
            "got: {:#?}",
            err
        );

        println!("{:?}", miette::Report::new(err));
    }

    fn slice_source_span<'a>(source: &'a NamedSource<String>, span: &'a SourceSpan) -> &'a str {
        let out = &source.inner().as_str()[span.offset()..(span.offset() + span.len())];

        println!("slice result: `{out}`");

        out
    }
}

#[cfg(test)]
mod tests2 {
    use pest::Parser;

    use crate::{
        expressions::Expression,
        identifier::{pkg_ident::PackageIdentifier, Identifier},
        parser::{common::handle_expression, package::ForComp, Rule, SspParser},
        types::Type,
        util::scope::Scope,
        writers::smt::exprs::{SmtLt, SmtLte},
    };

    use super::{MultiInstanceIndices, MultiInstanceIndicesGroup};

    #[test]
    #[ignore]
    fn example_smt_stuff() {
        let pkg_name = || "Foo".to_string();
        let mut scope = Scope::new();
        scope.enter();
        scope
            .declare(
                "n",
                crate::util::scope::Declaration::Identifier(Identifier::PackageIdentifier(
                    crate::identifier::pkg_ident::PackageIdentifier::Const(
                        crate::identifier::pkg_ident::PackageConstIdentifier {
                            pkg_name: pkg_name(),
                            name: "n".to_string(),
                            tipe: Type::Integer,
                            game_assignment: None,
                            pkg_inst_name: None,
                            game_name: None,
                            game_inst_name: None,
                            proof_name: None,
                        },
                    ),
                )),
            )
            .unwrap();

        let mut parse_expr = |text: &str| {
            handle_expression(
                SspParser::parse(Rule::expression, text)
                    .unwrap()
                    .next()
                    .unwrap(),
                &mut scope,
            )
            .unwrap()
        };

        let end = parse_expr("(n - 1)").map(|expr| match expr {
            Expression::Identifier(Identifier::Scalar(x)) => {
                Expression::Identifier(Identifier::Local(x))
            }
            x => x,
        });

        let indices_group = MultiInstanceIndicesGroup(vec![MultiInstanceIndices::new(vec![
            Expression::Identifier(Identifier::PackageIdentifier(
                PackageIdentifier::ImportsLoopVar(
                    crate::identifier::pkg_ident::PackageImportsLoopVarIdentifier {
                        pkg_name: pkg_name(),
                        name: "i".to_string(),
                        start: Box::new(parse_expr("0")),
                        end: Box::new(end),
                        start_comp: ForComp::Lte,
                        end_comp: ForComp::Lt,
                        pkg_inst_name: todo!(),
                        game_name: todo!(),
                        game_inst_name: None,
                        proof_name: None,
                    },
                ),
            )),
        ])]);

        let smt = indices_group.smt_check_total(
            vec![SmtLte(0, "x").into(), SmtLt("x", "n").into()],
            &["n"],
            "x",
        );

        for expr in smt {
            println!("{expr}")
        }

        println!("(check-sat)");
        println!("(get-model)")
    }
}
