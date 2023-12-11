use super::errors::{ErrorLocation, TypeCheckError};
use super::expression::{get_type, typify};
use super::scope::Scope;
use super::Type as ScopeType;

use crate::identifier::Identifier;
use crate::statement::{CodeBlock, Statement};
use crate::types::Type;

#[derive(Debug)]
pub struct TypedCodeBlock {
    pub expected_return_type: Type,
    pub block: CodeBlock,
}

impl TypedCodeBlock {
    pub fn typecheck(&self, scope: &mut Scope) -> Result<TypedCodeBlock, TypeCheckError> {
        let TypedCodeBlock {
            expected_return_type: ret_type,
            block,
        } = self;
        scope.enter();

        // unpack
        let block = &block.0;
        let mut new_block = vec![];

        for (i, stmt) in block.iter().enumerate() {
            //eprintln!("DEBUG typecheck_codeblock.for; {i}, {stmt:?}");
            match &*stmt {
                Statement::Abort(file_pos) => {
                    if i < block.len() - 1 {
                        return Err(TypeCheckError::MisplacedStatement(
                            ErrorLocation::FilePosition(file_pos.clone()),
                            "Abort found before end of code block!".to_string(),
                        ));
                    }

                    new_block.push(stmt.clone());
                }
                Statement::Return(Some(expr), file_pos) => {
                    let typed_expr = typify(expr, scope)?;
                    let expr_type = get_type(&typed_expr, scope)?;
                    if i < block.len() - 1 {
                        return Err(TypeCheckError::MisplacedStatement(
                            ErrorLocation::FilePosition(file_pos.clone()),
                            "Return found before end of code block!".to_string(),
                        ));
                    }
                    if expr_type != *ret_type {
                        return Err(TypeCheckError::TypeMismatch(
                            ErrorLocation::FilePosition(file_pos.clone()),
                            "return type does not match".to_owned(),
                            Some(expr.clone()),
                            expr_type,
                            ret_type.clone(),
                        ));
                    }
                    new_block.push(Statement::Return(Some(typed_expr), file_pos.clone()))
                }
                Statement::Return(None, file_pos) => {
                    if Type::Empty != *ret_type {
                        return Err(TypeCheckError::TypeMismatch(
                            ErrorLocation::FilePosition(file_pos.clone()),
                            "empty return in function that returns something".to_string(),
                            None,
                            Type::Empty,
                            ret_type.clone(),
                        ));
                    }

                    new_block.push(stmt.clone());
                }
                Statement::Assign(id, opt_idx, expr, file_pos) => {
                    //println!("scope: {:?}", scope);

                    let typed_expr = typify(expr, scope)?;
                    let expr_type = get_type(&typed_expr, scope)?;
                    if let Some(ScopeType::Type(id_type)) = scope.lookup(id) {
                        if let Some(idx) = opt_idx {
                            let typed_idx = typify(idx, scope)?;
                            let idx_type = get_type(&typed_idx, scope)?;
                            if let Type::Table(k, v) = id_type {
                                if *k != idx_type {
                                    return Err(TypeCheckError::TypeMismatch(
                                        ErrorLocation::FilePosition(file_pos.clone()),
                                        format!(
                                            "type used as index to table {:?} does not match",
                                            id
                                        ),
                                        Some(idx.clone()),
                                        idx_type,
                                        *k,
                                    ));
                                }
                                if Type::Maybe(v.clone()) != expr_type {
                                    return Err(TypeCheckError::TypeMismatch(
                                        ErrorLocation::FilePosition(file_pos.clone()),
                                        "value type of the table does not match".to_string(),
                                        None,
                                        *v,
                                        expr_type,
                                    ));
                                }
                            } else {
                                return Err(TypeCheckError::TypeMismatch(
                                    ErrorLocation::FilePosition(file_pos.clone()),
                                    "table access on non-table".to_string(),
                                    None,
                                    id_type,
                                    Type::Table(Box::new(idx_type), Box::new(expr_type)),
                                ));
                            }
                        } else if id_type != expr_type.clone() {
                            return Err(TypeCheckError::TypeMismatch(
                                ErrorLocation::FilePosition(file_pos.clone()),
                                format!("assigning to variable {:?} of different type", id),
                                Some(expr.clone()),
                                id_type,
                                expr_type,
                            ));
                        }
                    } else {
                        //eprintln!("DEBUG optidx is {opt_idx:?}; expr_type is {expr_type:?}");
                        if let Some(idxexpr) = opt_idx {
                            let idx_type = get_type(&idxexpr, scope)?;
                            let tabletipe = Type::Table(Box::new(idx_type), Box::new(expr_type));
                            scope
                                .declare(id.clone(), tabletipe)
                                .map_err(|err| TypeCheckError::new_scope_error(err, file_pos))?;
                        } else {
                            scope
                                .declare(id.clone(), expr_type)
                                .map_err(|err| TypeCheckError::new_scope_error(err, file_pos))?;
                        }
                    }

                    let opt_idx = if opt_idx.is_some() {
                        Some(typify(&opt_idx.clone().unwrap(), scope)?)
                    } else {
                        None
                    };

                    new_block.push(Statement::Assign(
                        id.clone(),
                        opt_idx,
                        typed_expr,
                        file_pos.clone(),
                    ));
                }
                Statement::Parse(idents, expr, file_pos) => {
                    let typed_expr = typify(expr, scope)?;
                    let expr_type = get_type(&typed_expr, scope)?;

                    if let Type::Tuple(types) = &expr_type {
                        if idents.len() != types.len() {
                            return Err(TypeCheckError::TypeMismatch(
                                ErrorLocation::FilePosition(file_pos.clone()),
                                format!(
                                    "parsing tuple {:?} of length {} into {} identifiers",
                                    expr,
                                    types.len(),
                                    idents.len()
                                ),
                                Some(expr.clone()),
                                Type::Empty,
                                expr_type.clone(),
                            ));
                        }

                        for (ident, t) in idents.iter().zip(types.iter()) {
                            if let Some(ScopeType::Type(t_ident)) = scope.lookup(ident) {
                                if &t_ident != t {
                                    return Err(TypeCheckError::TypeMismatch(
                                        ErrorLocation::FilePosition(file_pos.clone()),
                                        format!(
                                            "identifier {:?} in tuple parse has type {:?}, value is of type {:?}",
                                            ident,
                                            t_ident,
                                            t,
                                        ),
                                        Some(expr.clone()),
                                        Type::Empty,
                                        expr_type.clone(),
                                    ));
                                }
                            } else {
                                scope.declare(ident.clone(), t.clone()).map_err(|err| {
                                    TypeCheckError::new_scope_error(err, file_pos)
                                })?;
                            }
                        }
                    } else {
                        return Err(TypeCheckError::TypeMismatch(
                            ErrorLocation::FilePosition(file_pos.clone()),
                            format!(
                                "calling parse on {:?} of type {:?} which is not a tuple",
                                expr, expr_type
                            ),
                            Some(expr.clone()),
                            Type::Empty,
                            expr_type.clone(),
                        ));
                    }
                    new_block.push(Statement::Parse(
                        idents.clone(),
                        typed_expr,
                        file_pos.clone(),
                    ));
                }
                Statement::Sample(id, opt_idx, sample_id, sample_type, file_pos) => {
                    //println!("scope: {:?}", scope);

                    if let Some(ScopeType::Type(id_type)) = scope.lookup(id) {
                        if let Some(idx) = opt_idx {
                            let typed_idx = typify(idx, scope)?;
                            let idx_type = get_type(&typed_idx, scope)?;
                            if let Type::Table(k, v) = id_type {
                                if *k != idx_type {
                                    return Err(TypeCheckError::TypeMismatch(
                                        ErrorLocation::FilePosition(file_pos.clone()),
                                        format!(
                                            "type used as index to table {:?} does not match",
                                            id
                                        ),
                                        Some(idx.clone()),
                                        idx_type,
                                        *k,
                                    ));
                                }
                                if *v != *sample_type {
                                    return Err(TypeCheckError::TypeMismatch(
                                        ErrorLocation::FilePosition(file_pos.clone()),
                                        "value type of the table does not match".to_string(),
                                        None,
                                        *v,
                                        sample_type.clone(),
                                    ));
                                }
                            } else {
                                return Err(TypeCheckError::TypeMismatch(
                                    ErrorLocation::FilePosition(file_pos.clone()),
                                    "table access on non-table".to_string(),
                                    None,
                                    id_type,
                                    Type::Table(Box::new(idx_type), Box::new(sample_type.clone())),
                                ));
                            }
                        } else if id_type != sample_type.clone() {
                            return Err(TypeCheckError::TypeMismatch(
                                ErrorLocation::FilePosition(file_pos.clone()),
                                format!("sampling into variable {:?} of different type", id),
                                None,
                                id_type,
                                sample_type.clone(),
                            ));
                        }
                    } else {
                        scope
                            .declare(id.clone(), sample_type.clone())
                            .map_err(|err| TypeCheckError::new_scope_error(err, file_pos))?;
                    }

                    let opt_idx = if opt_idx.is_some() {
                        Some(typify(&opt_idx.clone().unwrap(), scope)?)
                    } else {
                        None
                    };

                    new_block.push(Statement::Sample(
                        id.clone(),
                        opt_idx,
                        *sample_id,
                        sample_type.clone(),
                        file_pos.clone(),
                    ));
                }
                Statement::InvokeOracle {
                    id,
                    opt_idx,
                    opt_dst_inst_idx,
                    name,
                    args,
                    target_inst_name,
                    tipe: _,
                    file_pos,
                } => {
                    println!(
                        "TODO: in typecheck/codeblock.rs, check that opt_dst_inst_idx is valid"
                    );
                    let oracle_entry = scope.lookup(&Identifier::new_scalar(name));
                    if oracle_entry.is_none() {
                        return Err(TypeCheckError::Undefined(
                            ErrorLocation::FilePosition(file_pos.clone()),
                            format!("no oracle with name {:} found", name),
                            Identifier::new_scalar(name),
                        ));
                    }

                    let oracle_entry = oracle_entry.unwrap().clone();

                    let (arg_types, ret_type) =
                        if let ScopeType::Oracle(arg_types, ret_type) = oracle_entry.clone() {
                            (arg_types, ret_type)
                        } else {
                            return Err(TypeCheckError::TypeMismatchVague(
                                ErrorLocation::FilePosition(file_pos.clone()),
                                format!("name {:} resolved to non-oracle type", name),
                                None,
                                Type::Empty,
                            ));
                        };

                    // 1. check that arg types match args
                    if args.len() != arg_types.len() {
                        return Err(TypeCheckError::TypeMismatch(
                                ErrorLocation::FilePosition(file_pos.clone()),
                            format!(
                                "oracle {} invocation on {:?} argument count mismatch. get {}, expected {}",
                                name,
                                target_inst_name,
                                args.len(),
                                arg_types.len()
                            ),
                            None,
                            Type::Empty,
                            Type::Fn(arg_types, Box::new(ret_type)),
                        ));
                    }

                    let mut typified_args = vec![];
                    for (i, arg) in args.iter().enumerate() {
                        let typified_arg = typify(arg, scope)?;
                        let t_arg = get_type(&typified_arg, scope)?;
                        if t_arg != arg_types[i] {
                            return Err(TypeCheckError::TypeMismatch(
                                ErrorLocation::FilePosition(file_pos.clone()),
                                format!("argument type mismatch at position {} at invocation of oracle {:}", i, name),
                                Some(arg.clone()),
                                t_arg,
                                arg_types[i].clone(),
                            ));
                        }

                        typified_args.push(typified_arg);
                    }
                    if let Some(ScopeType::Type(id_type)) = scope.lookup(id) {
                        if let Some(idx) = opt_idx {
                            let typed_idx = typify(idx, scope)?;
                            let idx_type = get_type(&typed_idx, scope)?;
                            if let Type::Table(k, v) = id_type {
                                if *k != idx_type {
                                    return Err(TypeCheckError::TypeMismatch(
                                        ErrorLocation::FilePosition(file_pos.clone()),
                                        format!(
                                            "type used as index to table {:?} does not match",
                                            id
                                        ),
                                        Some(idx.clone()),
                                        idx_type,
                                        *k,
                                    ));
                                }
                                if Type::Maybe(v.clone()) != ret_type {
                                    return Err(TypeCheckError::TypeMismatch(
                                        ErrorLocation::FilePosition(file_pos.clone()),
                                        "value type of the table does not match".to_string(),
                                        None,
                                        *v,
                                        ret_type,
                                    ));
                                }
                            } else {
                                return Err(TypeCheckError::TypeMismatch(
                                    ErrorLocation::FilePosition(file_pos.clone()),
                                    "table access on non-table".to_string(),
                                    None,
                                    id_type,
                                    Type::Table(Box::new(idx_type), Box::new(ret_type)),
                                ));
                            }
                        } else if id_type != ret_type.clone() {
                            return Err(TypeCheckError::TypeMismatch(
                                ErrorLocation::FilePosition(file_pos.clone()),
                                format!("sampling into variable {:?} of different type", id),
                                None,
                                id_type,
                                ret_type,
                            ));
                        }
                    } else {
                        scope
                            .declare(id.clone(), ret_type.clone())
                            .map_err(|err| TypeCheckError::new_scope_error(err, file_pos))?;
                    }

                    let opt_idx = if opt_idx.is_some() {
                        Some(typify(&opt_idx.clone().unwrap(), scope)?)
                    } else {
                        None
                    };

                    new_block.push(Statement::InvokeOracle {
                        id: id.clone(),
                        opt_idx: opt_idx.clone(),
                        opt_dst_inst_idx: opt_dst_inst_idx.clone(),
                        name: name.clone(),
                        args: typified_args,
                        target_inst_name: target_inst_name.clone(),
                        tipe: Some(ret_type.clone()),
                        file_pos: file_pos.clone(),
                    })
                }
                Statement::IfThenElse(expr, ifcode, elsecode, file_pos) => {
                    let typed_expr = typify(expr, scope)?;
                    let expr_type = get_type(&typed_expr, scope)?;

                    if expr_type != Type::Boolean {
                        return Err(TypeCheckError::TypeMismatch(
                            ErrorLocation::FilePosition(file_pos.clone()),
                            "expression used as condition in if-then-else is not boolean"
                                .to_string(),
                            Some(expr.clone()),
                            expr_type,
                            Type::Boolean,
                        ));
                    }

                    let typed_ifcode = TypedCodeBlock {
                        expected_return_type: ret_type.clone(),
                        block: ifcode.clone(),
                    }
                    .typecheck(scope)?;

                    let typed_elsecode = TypedCodeBlock {
                        expected_return_type: ret_type.clone(),
                        block: elsecode.clone(),
                    }
                    .typecheck(scope)?;

                    new_block.push(Statement::IfThenElse(
                        typed_expr,
                        typed_ifcode.block,
                        typed_elsecode.block,
                        file_pos.clone(),
                    ));
                }
                Statement::For(for_ident, lower_bound, upper_bound, body, file_pos) => {
                    let typed_lower_bound = typify(lower_bound, scope)?;
                    let typed_upper_bound = typify(upper_bound, scope)?;

                    let lower_bound_type = get_type(&typed_lower_bound, scope)?;
                    let upper_bound_type = get_type(&typed_upper_bound, scope)?;

                    if lower_bound_type != Type::Integer {
                        return Err(TypeCheckError::TypeMismatch(
                            ErrorLocation::FilePosition(file_pos.clone()),
                            "lower bound of for loop is not an integer".to_string(),
                            Some(lower_bound.clone()),
                            lower_bound_type,
                            Type::Integer,
                        ));
                    }

                    if upper_bound_type != Type::Integer {
                        return Err(TypeCheckError::TypeMismatch(
                            ErrorLocation::FilePosition(file_pos.clone()),
                            "upper bound of for loop is not an integer".to_string(),
                            Some(upper_bound.clone()),
                            upper_bound_type,
                            Type::Integer,
                        ));
                    }

                    scope.enter();
                    scope
                        .declare(for_ident.clone(), Type::Integer)
                        .map_err(|err| TypeCheckError::new_scope_error(err, file_pos))?;

                    let tcb = TypedCodeBlock {
                        expected_return_type: Type::Empty,
                        block: body.clone(),
                    };

                    match tcb.typecheck(scope) {
                        Ok(typed_body) => new_block.push(Statement::For(
                            for_ident.clone(),
                            typed_lower_bound,
                            typed_upper_bound,
                            typed_body.block,
                            file_pos.clone(),
                        )),
                        Err(e) => {
                            println!("{e:?}:\n{tcb:?}");
                            return Err(e);
                        }
                    }
                }
            }
        }

        scope.leave();
        Ok(TypedCodeBlock {
            block: CodeBlock(new_block),
            expected_return_type: ret_type.clone(),
        })
    }
}

/// unit tests for typing of (typed) code blocks
/// - Should honor the expected-return-type
///     return_none_fails, return_none_succeedes, return_wrong_type_fails, return_correcyt_type_succeedes
/// - Abort should be allowed
///     return_abort_succeedes
/// - Should follow branching
///     return_first_branch_wrong, return_second_branch_wrong, return_both_branch_correct, return_one_branch_aborts_correct
/// - Should check on (table-)assign
///     assign_succeedes_exists, assign_succeedes_new, assign_fails
#[cfg(test)]
mod test {
    use super::TypedCodeBlock;
    use crate::block;
    use crate::expressions::Expression;
    use crate::identifier::Identifier;
    use crate::statement::{CodeBlock, FilePosition, Statement};
    use crate::transforms::typecheck::{errors::TypeCheckError, scope::Scope};
    use crate::types::Type;

    #[test]
    fn return_none_fails() {
        let mut scope = Scope::new();
        let code = TypedCodeBlock {
            block: block! {
                Statement::Return(None, FilePosition::new("test_file.ssp".to_string(), 0, 1))
            },
            expected_return_type: Type::Integer,
        };
        let ret = code.typecheck(&mut scope);

        assert!(
            matches!(ret, Err(TypeCheckError::TypeMismatch(_, _, _, _, _))),
            "expected to fail with a TypeCheckError::TypeMismatch"
        );
    }

    #[test]
    fn return_none_succeedes() {
        let mut scope = Scope::new();
        let code = TypedCodeBlock {
            block: block! {
                Statement::Return(None, FilePosition::new("test_file.ssp".to_string(), 0, 1))
            },
            expected_return_type: Type::Empty,
        };
        let ret = code.typecheck(&mut scope);

        assert!(matches!(ret, Ok(_)), "Typecheck should succeed");
    }

    #[test]
    fn return_wrong_type_fails() {
        let mut scope = Scope::new();
        let code = TypedCodeBlock {
            block: block! {
                Statement::Return(Some(Expression::StringLiteral("test".to_string())), FilePosition::new("test_file.ssp".to_string(), 0, 1))
            },
            expected_return_type: Type::Integer,
        };
        let ret = code.typecheck(&mut scope);
        assert!(
            matches!(ret, Err(TypeCheckError::TypeMismatch(_, _, _, _, _))),
            "expected to fail with a TypeCheckError::TypeMismatch"
        );
    }

    #[test]
    fn return_correcyt_type_succeedes() {
        let mut scope = Scope::new();
        let code = TypedCodeBlock {
            block: block! {
                Statement::Return(Some(Expression::IntegerLiteral("23".to_string())), FilePosition::new("test_file.ssp".to_string(), 0, 1))
            },
            expected_return_type: Type::Integer,
        };
        let ret = code.typecheck(&mut scope);

        assert!(matches!(ret, Ok(_)), "Typecheck should succeed");
    }

    #[test]
    fn return_abort_succeedes() {
        let mut scope = Scope::new();
        let code = TypedCodeBlock {
            block: block! {
                Statement::Abort( FilePosition::new("test_file.ssp".to_string(), 0, 1))
            },
            expected_return_type: Type::Integer,
        };
        let ret = code.typecheck(&mut scope);

        assert!(matches!(ret, Ok(_)), "Typecheck should succeed");
    }

    #[test]
    fn return_first_branch_wrong() {
        let mut scope = Scope::new();
        let code = TypedCodeBlock {
            block: block! {
            Statement::IfThenElse(
                Expression::new_equals(vec![&(Expression::StringLiteral("23".to_string())),
                                            &(Expression::StringLiteral("23".to_string()))]),
                block!{
                    Statement::Return(Some(Expression::StringLiteral("23".to_string())),FilePosition::new("test_file.ssp".to_string(), 0, 1))
                },
                block!{
                    Statement::Return(Some(Expression::IntegerLiteral("23".to_string())),FilePosition::new("test_file.ssp".to_string(), 0, 1))
                },FilePosition::new("test_file.ssp".to_string(), 0, 1) )
            },
            expected_return_type: Type::Integer,
        };
        let ret = code.typecheck(&mut scope);

        assert!(
            matches!(ret, Err(TypeCheckError::TypeMismatch(_, _, _, _, _))),
            "expected to fail with a TypeCheckError::TypeMismatch"
        );
    }

    #[test]
    fn return_second_branch_wrong() {
        let mut scope = Scope::new();
        let code = TypedCodeBlock {
            block: block! {
            Statement::IfThenElse(
                Expression::new_equals(vec![&(Expression::StringLiteral("23".to_string())),
                                            &(Expression::StringLiteral("23".to_string()))]),
                block!{
                    Statement::Return(Some(Expression::IntegerLiteral("23".to_string())),FilePosition::new("test_file.ssp".to_string(), 0, 1))
                },
                block!{
                    Statement::Return(Some(Expression::StringLiteral("23".to_string())),FilePosition::new("test_file.ssp".to_string(), 0, 1))
                },FilePosition::new("test_file.ssp".to_string(), 0, 1))
            },
            expected_return_type: Type::Integer,
        };
        let ret = code.typecheck(&mut scope);

        assert!(
            matches!(ret, Err(TypeCheckError::TypeMismatch(_, _, _, _, _))),
            "expected to fail with a TypeCheckError::TypeMismatch"
        );
    }

    #[test]
    fn return_both_branch_correct() {
        let mut scope = Scope::new();
        let code = TypedCodeBlock {
            block: block! {
            Statement::IfThenElse(
                Expression::new_equals(vec![&(Expression::StringLiteral("23".to_string())),
                                            &(Expression::StringLiteral("23".to_string()))]),
                block!{
                    Statement::Return(Some(Expression::IntegerLiteral("23".to_string())),FilePosition::new("test_file.ssp".to_string(), 0, 1))
                },
                block!{
                    Statement::Return(Some(Expression::IntegerLiteral("23".to_string())),FilePosition::new("test_file.ssp".to_string(), 0, 1))
                },FilePosition::new("test_file.ssp".to_string(), 0, 1))
            },
            expected_return_type: Type::Integer,
        };
        let ret = code.typecheck(&mut scope);
        assert!(matches!(ret, Ok(_)), "Typecheck should succeed");
    }

    #[test]
    fn return_one_branch_aborts_correct() {
        let mut scope = Scope::new();
        let code = TypedCodeBlock {
            block: block! {
            Statement::IfThenElse(
                Expression::new_equals(vec![&(Expression::StringLiteral("23".to_string())),
                                            &(Expression::StringLiteral("23".to_string()))]),
                block!{
                    Statement::Return(Some(Expression::IntegerLiteral("23".to_string())),FilePosition::new("test_file.ssp".to_string(), 0, 1))
                },
                block!{
                    Statement::Abort(FilePosition::new("test_file.ssp".to_string(), 0, 1))
                },FilePosition::new("test_file.ssp".to_string(), 0, 1))
            },
            expected_return_type: Type::Integer,
        };
        let ret = code.typecheck(&mut scope);

        assert!(matches!(ret, Ok(_)), "Typecheck should succeed");
    }

    #[test]
    fn assign_fails() {
        let mut scope = Scope::new();
        scope.enter();

        scope
            .declare(Identifier::Local("test".to_string()), Type::Integer)
            .unwrap();
        let code = TypedCodeBlock {
            block: block! {
                            Statement::Assign(
                                Identifier::Local("test".to_string()),
                                None,
                                Expression::StringLiteral("42".to_string()),
            FilePosition::new("test_file.ssp".to_string(), 0, 1))
                        },
            expected_return_type: Type::Empty,
        };
        let ret = code.typecheck(&mut scope);

        assert!(
            matches!(ret, Err(TypeCheckError::TypeMismatch(_, _, _, _, _))),
            "expected to fail with a TypeCheckError::TypeMismatch"
        );
    }

    #[test]
    fn assign_succeedes_exists() {
        let mut scope = Scope::new();
        scope.enter();
        scope
            .declare(Identifier::Local("test".to_string()), Type::Integer)
            .unwrap();
        let code = TypedCodeBlock {
            block: block! {
                Statement::Assign(Identifier::Local("test".to_string()), None, Expression::IntegerLiteral("42".to_string()),
            FilePosition::new("test_file.ssp".to_string(), 0, 1))
            },
            expected_return_type: Type::Empty,
        };
        let ret = code.typecheck(&mut scope);

        assert!(matches!(ret, Ok(_)), "Typecheck should succeed");
    }

    #[test]
    fn assign_succeedes_new() {
        let mut scope = Scope::new();
        scope.enter();
        let code = TypedCodeBlock {
            block: block! {
                Statement::Assign(Identifier::Local("test".to_string()), None, Expression::IntegerLiteral("42".to_string()),
            FilePosition::new("test_file.ssp".to_string(), 0, 1))
            },
            expected_return_type: Type::Empty,
        };
        let ret = code.typecheck(&mut scope);

        assert!(matches!(ret, Ok(_)), "Typecheck should succeed");
    }

    #[test]
    fn table_assign_succeedes() {
        let mut scope = Scope::new();
        scope.enter();
        scope
            .declare(
                Identifier::Local("test".to_string()),
                Type::Table(Box::new(Type::Integer), Box::new(Type::String)),
            )
            .unwrap();
        let code = TypedCodeBlock {
            block: block! {
                Statement::Assign(Identifier::Local("test".to_string()),
                                       Some(Expression::IntegerLiteral("42".to_string())),
                                       Expression::Some(Box::new(Expression::StringLiteral("42".to_string()))),
            FilePosition::new("test_file.ssp".to_string(), 0, 1))
            },
            expected_return_type: Type::Empty,
        };
        let ret = code.typecheck(&mut scope);

        if let Err(ref e) = ret {
            println!("error: {:#?}", e);
        }

        assert!(matches!(ret, Ok(_)), "Typecheck should succeed");
    }

    #[test]
    fn table_assign_wrong_index_type() {
        let mut scope = Scope::new();
        scope.enter();
        scope
            .declare(
                Identifier::Local("test".to_string()),
                Type::Table(Box::new(Type::Integer), Box::new(Type::String)),
            )
            .unwrap();
        let code = TypedCodeBlock {
            block: block! {
                Statement::Assign(Identifier::Local("test".to_string()),
                                       Some(Expression::StringLiteral("42".to_string())),
                                       Expression::StringLiteral("42".to_string()),
            FilePosition::new("test_file.ssp".to_string(), 0, 1))
            },
            expected_return_type: Type::Empty,
        };
        let ret = code.typecheck(&mut scope);
        assert!(
            matches!(ret, Err(TypeCheckError::TypeMismatch(_, _, _, _, _))),
            "expected to fail with a TypeCheckError::TypeMismatch"
        );
    }

    #[test]
    fn table_assign_wrong_value() {
        let mut scope = Scope::new();
        scope.enter();
        scope
            .declare(
                Identifier::Local("test".to_string()),
                Type::Table(Box::new(Type::Integer), Box::new(Type::String)),
            )
            .unwrap();
        let code = TypedCodeBlock {
            block: block! {
                Statement::Assign(Identifier::Local("test".to_string()),
                                       Some(Expression::IntegerLiteral("42".to_string())),
                                       Expression::IntegerLiteral("42".to_string()),
            FilePosition::new("test_file.ssp".to_string(), 0, 1))
            },
            expected_return_type: Type::Empty,
        };
        let ret = code.typecheck(&mut scope);
        assert!(
            matches!(ret, Err(TypeCheckError::TypeMismatch(_, _, _, _, _))),
            "expected to fail with a TypeCheckError::TypeMismatch"
        );
    }

    #[test]
    fn table_assign_not_table() {
        let mut scope = Scope::new();
        scope.enter();
        scope
            .declare(Identifier::Local("test".to_string()), Type::Integer)
            .unwrap();
        let code = TypedCodeBlock {
            block: block! {
                Statement::Assign(Identifier::Local("test".to_string()),
                                       Some(Expression::IntegerLiteral("42".to_string())),
                                       Expression::IntegerLiteral("42".to_string()),
            FilePosition::new("test_file.ssp".to_string(), 0, 1))
            },
            expected_return_type: Type::Empty,
        };
        let ret = code.typecheck(&mut scope);
        assert!(
            matches!(ret, Err(TypeCheckError::TypeMismatch(_, _, _, _, _))),
            "expected to fail with a TypeCheckError::TypeMismatch"
        );
    }

    #[test]
    fn table_assign_undeclared() {
        let mut scope = Scope::new();
        scope.enter();
        let code = TypedCodeBlock {
            block: block! {
                Statement::Assign(
                    Identifier::Local("test".to_string()),
                                       Some(Expression::IntegerLiteral("42".to_string())),
                                       Expression::IntegerLiteral("42".to_string()),
                                    FilePosition::new("test_file.ssp".to_string(), 0, 1))
            },
            expected_return_type: Type::Empty,
        };
        let ret = code.typecheck(&mut scope);

        if let Err(ref e) = ret {
            println!("error: {:#?}", e);
        }

        assert!(matches!(ret, Ok(_)), "Typecheck should succeed");
    }
}
