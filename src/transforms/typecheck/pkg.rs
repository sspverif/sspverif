use super::errors::TypeCheckError;
use super::oracledef::typecheck_odef;
use super::scope::Scope;

use crate::identifier::Identifier;
use crate::package::{OracleDef, OracleSig, Package};

pub fn typecheck_pkg(pkg: &Package, scope: &mut Scope) -> Result<Package, TypeCheckError> {
    let Package {
        params,
        state,
        oracles,
        ..
    } = pkg;

    scope.enter();
    for (name, ntipe) in params {
        scope.declare(Identifier::new_scalar(name), ntipe.clone())?;
    }
    for (name, ntipe) in state {
        scope.declare(Identifier::new_scalar(name), ntipe.clone())?;
    }

    let mut typed_oracles = vec![];

    for oracle in oracles {
        // kept around so we don't have to get it out when we want to debug it next time
        let OracleDef {
            sig: OracleSig {
                name: _oracle_name, ..
            },
            ..
        } = &oracle;
        //eprintln!("DEBUG typecheck_pkg: checking {pkg_name}.{oracle_name}");
        typed_oracles.push(typecheck_odef(oracle, scope)?);
    }

    scope.leave();
    Ok(Package {
        oracles: typed_oracles,
        ..pkg.clone()
    })
}
