use crate::expressions::Expression;
use crate::types::Type;

use super::errors::{ErrorLocation, ExpressionResult, TypeCheckError, TypeResult};
use super::scope::Scope;

pub fn get_type(expr: &Expression, scope: &Scope) -> TypeResult {
    Ok(match expr {
        Expression::Typed(t, _) => t.clone(),
        _ => get_type(&typify(expr, scope)?, scope)?,
    })
}

pub fn typify(expr: &Expression, scope: &Scope) -> ExpressionResult {
    //eprintln!("DEBUG typify {expr:?}");
    match expr {
        Expression::Typed(_, _) => Ok(expr.clone()),
        Expression::StringLiteral(_) => Ok(Expression::Typed(Type::String, expr.clone().into())),
        Expression::IntegerLiteral(_) => Ok(Expression::Typed(Type::Integer, expr.clone().into())),
        Expression::BooleanLiteral(_) => Ok(Expression::Typed(Type::Boolean, expr.clone().into())),
        Expression::Equals(exprs) => {
            let outer_expr = expr;
            let mut exprs_ = vec![];
            let mut t_opt: Option<Type> = None;

            for expr in exprs {
                exprs_.push(typify(expr, scope)?);

                match &t_opt {
                    None => {
                        t_opt = Some(get_type(expr, scope)?);
                    }
                    Some(t) => {
                        let t_found = get_type(expr, scope)?;
                        if t != &t_found {
                            return Err(TypeCheckError::TypeMismatch(
                                ErrorLocation::Unknown,
                                "equality compares expression of different type".to_string(),
                                Some(outer_expr.clone()),
                                t_found,
                                t.clone(),
                            ));
                        }
                    }
                }
            }
            Ok(Expression::Typed(
                Type::Boolean,
                Box::new(Expression::Equals(exprs_)),
            ))
        }
        Expression::Tuple(elems) => {
            let mut types = vec![];
            let mut elems_ = vec![];

            for elem in elems {
                let elem_ = typify(elem, scope)?;
                let t = get_type(&elem_, scope)?;
                types.push(t);
                elems_.push(elem_);
            }

            Ok(Expression::Typed(
                Type::Tuple(types),
                Box::new(Expression::Tuple(elems_)),
            ))
        }
        Expression::Some(v) => Ok(Expression::Typed(
            Type::Maybe(Box::new(get_type(v, scope)?)),
            Box::new(Expression::Some(Box::new(typify(v, scope)?))),
        )),
        Expression::None(t) => Ok(Expression::Typed(
            Type::Maybe(Box::new(t.clone())),
            Box::new(Expression::None(t.clone())),
        )),
        Expression::Unwrap(v) => {
            //eprintln!("DEBUG typify Unwrap({v:?})");

            match &**v {
                Expression::Some(inner) => Ok(Expression::Typed(
                    get_type(inner, scope)?,
                    Box::new(Expression::Unwrap(Box::new(typify(v, scope)?))),
                )),
                Expression::None(t) => Ok(Expression::Typed(t.clone(), Box::new(expr.clone()))),
                Expression::Identifier(id) => {
                    if let Some(t) = scope.lookup(id) {
                        //eprintln!("DEBUG typify Unwrap Identifier {id:?}, found type {t:?}");
                        if let Type::Maybe(inner_t) = t {
                            Ok(Expression::Typed(*inner_t, Box::new(expr.clone())))
                        } else {
                            Err(TypeCheckError::ShouldBeMaybe(
                                ErrorLocation::Unknown,
                                "type error: Unwrap contains identifier with non-Maybe type"
                                    .to_string(),
                                expr.clone(),
                                t.clone(),
                            ))
                        }
                    } else {
                        Err(TypeCheckError::Undefined(
                            ErrorLocation::Unknown,
                            "type error: Unwrap contains identifier that is not in scope"
                                .to_string(),
                            id.clone(),
                        ))
                    }
                }
                Expression::TableAccess(id, _) => {
                    if let Some(Type::Table(_, t_val)) = scope.lookup(id) {
                        //eprintln!("DEBUG typify Unwrap of TableAccess on Identifier({id:?}) that maps to {t_val:?}");
                        Ok(Expression::Typed(*t_val, Box::new(expr.clone())))
                    } else {
                        Err(TypeCheckError::Undefined(
                            ErrorLocation::Unknown,
                            "type error: Unwrap contains access to table that is not in scope"
                                .to_string(),
                            id.clone(),
                        ))
                    }
                }
                _ => Err(TypeCheckError::ShouldBeMaybe(
                    ErrorLocation::Unknown,
                    "type error: Unwrap contains identifier with non-Maybe type".to_string(),
                    expr.clone(),
                    get_type(v, scope)?,
                )),
            }
            /*
            if let Expression::Some(inner) = &**v {
            } else {
            }
            */
        }
        Expression::Neg(v) => {
            let t = get_type(v, scope)?;
            if t == Type::Integer || matches!(t, Type::AddiGroupEl(_)) {
                Ok(Expression::Typed(
                    t,
                    Box::new(Expression::Neg(Box::new(typify(v, scope)?))),
                ))
            } else {
                Err(TypeCheckError::TypeMismatchVague(
                    ErrorLocation::Unknown,
                    "negation only allowed on integers and additive group elements".to_string(),
                    Some(expr.clone()),
                    t,
                ))
            }
        }
        Expression::Not(v) => {
            let t = get_type(v, scope)?;
            if t != Type::Boolean {
                return Err(TypeCheckError::TypeMismatch(
                    ErrorLocation::Unknown,
                    "Not only allowed on boolean expressions".to_string(),
                    Some(expr.clone()),
                    t,
                    Type::Boolean,
                ));
            }

            Ok(Expression::Typed(
                t,
                Box::new(Expression::Not(Box::new(typify(v, scope)?))),
            ))
        }
        Expression::Inv(v) => {
            let t = get_type(v, scope)?;
            if matches!(t, Type::MultGroupEl(_)) {
                Ok(Expression::Typed(
                    t,
                    Box::new(Expression::Inv(Box::new(typify(v, scope)?))),
                ))
            } else {
                Err(TypeCheckError::TypeMismatchVague(
                    ErrorLocation::Unknown,
                    "inverse only allowed on multiplicative group elements".to_string(),
                    Some(expr.clone()),
                    t,
                ))
            }
        }
        Expression::Add(left, right) => {
            let t_left = get_type(left, scope)?;
            let t_right = get_type(right, scope)?;

            let same_type = t_left == t_right;
            let left_is_int = t_left == Type::Integer;
            let left_is_age = matches!(t_left, Type::AddiGroupEl(_));

            if same_type && (left_is_int || left_is_age) {
                Ok(Expression::Typed(
                    t_left,
                    Box::new(Expression::Add(
                        Box::new(typify(left, scope)?),
                        Box::new(typify(right, scope)?),
                    )),
                ))
            } else if !(left_is_int || left_is_age) {
                Err(TypeCheckError::TypeMismatchVague(
                    ErrorLocation::Unknown,
                    "addition only works for ints and additive group elements".to_string(),
                    Some(*left.clone()),
                    t_left,
                ))
            } else {
                Err(TypeCheckError::TypeMismatch(
                    ErrorLocation::Unknown,
                    "added types don't match".to_string(),
                    Some(*right.clone()),
                    t_right,
                    t_left,
                ))
            }
        }
        Expression::Mul(left, right) => {
            let t_left = get_type(left, scope)?;
            let t_right = get_type(right, scope)?;

            let same_type = t_left == t_right;
            let left_is_int = t_left == Type::Integer;
            let left_is_mge = matches!(t_left, Type::MultGroupEl(_));
            let right_is_age = matches!(t_right, Type::AddiGroupEl(_));

            #[allow(clippy::collapsible_else_if)]
            if same_type {
                if left_is_int || left_is_mge {
                    Ok(Expression::Typed(
                        t_left,
                        Box::new(Expression::Mul(
                            Box::new(typify(left, scope)?),
                            Box::new(typify(right, scope)?),
                        )),
                    ))
                } else {
                    Err(TypeCheckError::TypeMismatchVague(
                        ErrorLocation::Unknown,
                        "multiplication only works for ints and multiplicative group elements"
                            .to_string(),
                        Some(*left.clone()),
                        t_left,
                    ))
                }
            } else {
                if left_is_int && right_is_age {
                    Ok(Expression::Typed(
                        t_right,
                        Box::new(Expression::Mul(
                            Box::new(typify(left, scope)?),
                            Box::new(typify(right, scope)?),
                        )),
                    ))
                } else {
                    Err(TypeCheckError::TypeMismatchVague(
                        ErrorLocation::Unknown,
                        "multiplication only works for ints and multiplicative group elements"
                            .to_string(),
                        Some(*left.clone()),
                        t_left,
                    ))
                }
            }
        }
        Expression::Sub(left, right) => {
            let t_left = get_type(left, scope)?;
            let t_right = get_type(right, scope)?;

            let same_type = t_left == t_right;
            let left_is_int = t_left == Type::Integer;
            let left_is_age = matches!(t_left, Type::AddiGroupEl(_));

            if (left_is_int || left_is_age) && same_type {
                Ok(Expression::Typed(
                    t_left,
                    Box::new(Expression::Sub(
                        Box::new(typify(left, scope)?),
                        Box::new(typify(right, scope)?),
                    )),
                ))
            } else if !(left_is_int || left_is_age) {
                Err(TypeCheckError::TypeMismatchVague(
                    ErrorLocation::Unknown,
                    "subtraction only works for ints and additive group elements".to_string(),
                    Some(*left.clone()),
                    t_left,
                ))
            } else {
                Err(TypeCheckError::TypeMismatch(
                    ErrorLocation::Unknown,
                    "subtracted types don't match".to_string(),
                    Some(*right.clone()),
                    t_right,
                    t_left,
                ))
            }
        }
        Expression::Div(left, right) => {
            let t_left = get_type(left, scope)?;
            let t_right = get_type(right, scope)?;

            if t_left != Type::Integer {
                Err(TypeCheckError::TypeMismatch(
                    ErrorLocation::Unknown,
                    "divisor/numerator is not an int".to_string(),
                    Some(*left.clone()),
                    t_left,
                    Type::Integer,
                ))
            } else if t_right != Type::Integer {
                Err(TypeCheckError::TypeMismatch(
                    ErrorLocation::Unknown,
                    "dividend/denominator is not an int".to_string(),
                    Some(*right.clone()),
                    t_right,
                    Type::Integer,
                ))
            } else {
                Ok(Expression::Typed(
                    t_left,
                    Box::new(Expression::Div(
                        Box::new(typify(left, scope)?),
                        Box::new(typify(right, scope)?),
                    )),
                ))
            }
        }
        Expression::Pow(base, exp) => {
            let t_base = get_type(base, scope)?;
            let t_exp = get_type(exp, scope)?;

            let base_is_int = t_base == Type::Integer;
            let exp_is_int = t_exp == Type::Integer;
            let base_is_mge = matches!(t_base, Type::MultGroupEl(_));

            if exp_is_int {
                if base_is_int || base_is_mge {
                    Ok(Expression::Typed(
                        t_base,
                        Box::new(Expression::Pow(
                            Box::new(typify(base, scope)?),
                            Box::new(typify(exp, scope)?),
                        )),
                    ))
                } else {
                    Err(TypeCheckError::TypeMismatchVague(
                        ErrorLocation::Unknown,
                        "base for exponentiation needs to be int or multiplicative group element"
                            .to_string(),
                        Some(*base.clone()),
                        t_base,
                    ))
                }
            } else {
                Err(TypeCheckError::TypeMismatch(
                    ErrorLocation::Unknown,
                    "exponent is not an int".to_string(),
                    Some(*exp.clone()),
                    t_exp,
                    Type::Integer,
                ))
            }
        }

        Expression::Mod(num, modulus) => {
            let t_num = get_type(num, scope)?;
            let t_mod = get_type(modulus, scope)?;

            if t_num != Type::Integer {
                Err(TypeCheckError::TypeMismatch(
                    ErrorLocation::Unknown,
                    "modulo'd number is not an int".to_string(),
                    Some(*num.clone()),
                    t_num,
                    Type::Integer,
                ))
            } else if t_mod != Type::Integer {
                Err(TypeCheckError::TypeMismatch(
                    ErrorLocation::Unknown,
                    "modulus is not an int".to_string(),
                    Some(*modulus.clone()),
                    t_mod,
                    Type::Integer,
                ))
            } else {
                Ok(Expression::Typed(
                    t_num,
                    Box::new(Expression::Pow(
                        Box::new(typify(num, scope)?),
                        Box::new(typify(modulus, scope)?),
                    )),
                ))
            }
        }

        Expression::Xor(vs) => {
            let mut vs_ = vec![];
            // TODO bit strings
            for v in vs {
                let v_ = typify(v, scope)?;
                let t_v = get_type(&v_, scope)?;
                if t_v != Type::Boolean {
                    return Err(TypeCheckError::TypeMismatch(
                        ErrorLocation::Unknown,
                        "xor'd value is not a bool".to_string(),
                        Some(v.clone()),
                        t_v,
                        Type::Boolean,
                    ));
                }

                vs_.push(v_);
            }

            Ok(Expression::Typed(
                Type::Boolean,
                Box::new(Expression::Xor(vs_)),
            ))
        }

        Expression::And(vs) => {
            let mut vs_ = vec![];
            // TODO bit strings
            for v in vs {
                let v_ = typify(v, scope)?;
                let t_v = get_type(&v_, scope)?;
                if t_v != Type::Boolean {
                    return Err(TypeCheckError::TypeMismatch(
                        ErrorLocation::Unknown,
                        "and'd value is not a bool".to_string(),
                        Some(v.clone()),
                        t_v,
                        Type::Boolean,
                    ));
                }
                vs_.push(v_);
            }

            Ok(Expression::Typed(
                Type::Boolean,
                Box::new(Expression::And(vs_)),
            ))
        }

        Expression::Or(vs) => {
            let mut vs_ = vec![];
            // TODO bit strings
            for v in vs {
                let v_ = typify(v, scope)?;
                let t_v = get_type(&v_, scope)?;
                if t_v != Type::Boolean {
                    return Err(TypeCheckError::TypeMismatch(
                        ErrorLocation::Unknown,
                        "or'd value is not a bool".to_string(),
                        Some(v.clone()),
                        t_v,
                        Type::Boolean,
                    ));
                }
                vs_.push(v_);
            }

            Ok(Expression::Typed(
                Type::Boolean,
                Box::new(Expression::Or(vs_)),
            ))
        }

        Expression::FnCall(id, args) => {
            if let Some(Type::Fn(arg_types, ret_type)) = scope.lookup(id) {
                // 1. check that arg types match args
                if args.len() != arg_types.len() {
                    return Err(TypeCheckError::TypeMismatch(
                        ErrorLocation::Unknown,
                        format!(
                            "type error: argument count mismatch. get {}, expected {}",
                            args.len(),
                            arg_types.len()
                        ),
                        Some(expr.clone()),
                        Type::Fn(
                            args.iter()
                                .map(|arg| get_type(arg, scope))
                                .collect::<Result<Vec<_>, _>>()?,
                            ret_type.clone(),
                        ),
                        Type::Fn(arg_types, ret_type),
                    ));
                }

                let mut typified_args = vec![];

                for (i, arg) in args.iter().enumerate() {
                    let typified_arg = typify(arg, scope)?;
                    let arg_type = get_type(&typified_arg, scope)?;
                    if arg_type != arg_types[i] {
                        return Err(TypeCheckError::TypeMismatch(
                            ErrorLocation::Unknown,
                            format!(
                                "argument type mismatch at position {} when calling function {:}",
                                i,
                                id.ident()
                            ),
                            Some(arg.clone()),
                            arg_type,
                            arg_types[i].clone(),
                        ));
                    }

                    typified_args.push(typified_arg);
                }

                // 2. return ret type
                Ok(Expression::Typed(
                    *ret_type,
                    Box::new(Expression::FnCall(id.clone(), typified_args)),
                ))
            } else {
                Err(TypeCheckError::Undefined(
                    ErrorLocation::Unknown,
                    "function not found in scope".to_string(),
                    id.clone(),
                ))
            }
        }

        Expression::Identifier(id) => {
            if let Some(t) = scope.lookup(id) {
                Ok(Expression::Typed(
                    t,
                    Box::new(Expression::Identifier(id.clone())),
                ))
            } else {
                Err(TypeCheckError::Undefined(
                    ErrorLocation::Unknown,
                    "identifier not found in scope".to_owned(),
                    id.clone(),
                ))
            }
        }
        Expression::TableAccess(id, expr) => match scope.lookup(id) {
            Some(Type::Table(t_idx, t_val)) => {
                let expr_ = typify(expr, scope)?;
                let t_expr = get_type(&expr_, scope)?;
                if *t_idx == t_expr {
                    Ok(Expression::Typed(
                        Type::Maybe(t_val),
                        Box::new(Expression::TableAccess(id.clone(), Box::new(expr_))),
                    ))
                } else {
                    Err(TypeCheckError::TypeMismatch(
                        ErrorLocation::Unknown,
                        "unexpected index type".to_string(),
                        Some(*expr.clone()),
                        t_expr,
                        *t_idx,
                    ))
                }
            }
            Some(t) => Err(TypeCheckError::TypeMismatchVague(
                ErrorLocation::Unknown,
                "table access on non-table value".to_string(),
                Some(id.to_expression()),
                t,
            )),
            _ => Err(TypeCheckError::Undefined(
                ErrorLocation::Unknown,
                "accessing undefined table".to_string(),
                id.clone(),
            )),
        },
        Expression::List(exprs) => {
            let outer_expr = expr;
            let mut exprs_ = vec![];
            let mut inner: Option<Type> = None;

            for expr in exprs {
                exprs_.push(typify(expr, scope)?);

                match &inner {
                    None => {
                        inner = Some(get_type(expr, scope)?);
                    }
                    Some(t) => {
                        let t_found = get_type(expr, scope)?;
                        if t != &t_found {
                            return Err(TypeCheckError::TypeMismatch(
                                ErrorLocation::Unknown,
                                "list contains expressions of different types".to_string(),
                                Some(outer_expr.clone()),
                                t_found,
                                t.clone(),
                            ));
                        }
                    }
                }
            }
            Ok(Expression::Typed(
                Type::List(Box::new(inner.unwrap())),
                Box::new(Expression::List(exprs_)),
            ))
        }
        Expression::Set(exprs) => {
            let outer_expr = expr;
            let mut exprs_ = vec![];
            let mut inner: Option<Type> = None;

            for expr in exprs {
                exprs_.push(typify(expr, scope)?);

                match &inner {
                    None => {
                        inner = Some(get_type(expr, scope)?);
                    }
                    Some(t) => {
                        let t_found = get_type(expr, scope)?;
                        if t != &t_found {
                            return Err(TypeCheckError::TypeMismatch(
                                ErrorLocation::Unknown,
                                "set contains expressions of different types".to_string(),
                                Some(outer_expr.clone()),
                                t_found,
                                t.clone(),
                            ));
                        }
                    }
                }
            }
            Ok(Expression::Typed(
                Type::List(Box::new(inner.unwrap())),
                Box::new(Expression::Set(exprs_)),
            ))
        }
        _ => {
            panic!("get_type not implemented for {:#?}", expr);
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::scope::Scope;
    use super::typify;
    use crate::expressions::Expression;
    use crate::identifier::Identifier;
    use crate::types::Type;

    #[test]
    fn typeifying_fncall_argumentcount_error() {
        // Should produce an error (and not an infinite recursion)
        let mut scope = Scope::new();
        scope.enter();
        scope
            .declare(
                Identifier::Scalar("test".to_string()),
                Type::Fn(vec![Type::Integer, Type::Integer], Box::new(Type::Integer)),
            )
            .unwrap();
        let typeify_res = typify(
            &Expression::FnCall(
                Identifier::Scalar("test".to_string()),
                vec![Expression::IntegerLiteral("12".to_string())],
            ),
            &scope,
        );
        assert!(typeify_res.is_err());
        // assert_matches crate?
        // let typeerror = typeify_res.unwrap_err();
        // assert_eq!(typeerror, TypeCheckError::TypeMissmatch(_, _, _, _, _))
    }
}
