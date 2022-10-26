use super::{
    assumption::ResolvedAssumption,
    equivalence::{Equivalence, ResolvedEquivalence},
    error::{Error, Result},
    reduction::{Reduction, ResolvedReduction},
    Assumption, Project,
};

impl Project {
    // i is passed for error reporting only
    pub fn resolve_reduction(&self, reduction: &Reduction, i: usize) -> Result<ResolvedReduction> {
        let left = self
            .get_game(&reduction.left)
            .ok_or(Error::UndefinedGame(
                reduction.left.clone(),
                format!("in left of reduction {i}"),
            ))?
            .clone();

        let right = self
            .get_game(&reduction.right)
            .ok_or(Error::UndefinedGame(
                reduction.right.clone(),
                format!("in right of reduction {i}"),
            ))?
            .clone();

        let assumption =
            self.get_assumption(&reduction.assumption)
                .ok_or(Error::UndefinedAssumption(
                    reduction.assumption.clone(),
                    format!("in reduction {i}"),
                ))?;

        let assumption = self.resolve_assumption(assumption)?;

        let assumption_name = reduction.assumption.clone();

        let leftmap : Result<Vec<_>> = reduction.leftmap.iter().map(|(from, to)|{
            let gameindex = left.pkgs.iter().position(|pkg| &pkg.name == from).ok_or(Error::UndefinedMapping(
                    from.clone(),
                    format!("in reduction {i}, left game")))?;
            let assumptionindex = assumption.left.pkgs.iter().position(|pkg| &pkg.name == to).ok_or(Error::UndefinedMapping(
                    to.clone(),
                    format!("in reduction {i}, left assumption")))?;
            Ok((gameindex, assumptionindex))
        }).collect();
        let leftmap = leftmap?;

        let rightmap : Result<Vec<_>> = reduction.rightmap.iter().map(|(from, to)|{
            let gameindex = right.pkgs.iter().position(|pkg| &pkg.name == from).ok_or(Error::UndefinedMapping(
                    from.clone(),
                    format!("in reduction {i}, right game")))?;
            let assumptionindex = assumption.right.pkgs.iter().position(|pkg| &pkg.name == to).ok_or(Error::UndefinedMapping(
                    to.clone(),
                    format!("in reduction {i}, right assumption")))?;
            Ok((gameindex, assumptionindex))
        }).collect();
        let rightmap = rightmap?;

        
        Ok(ResolvedReduction {
            left,
            right,
            assumption,
            assumption_name,
            leftmap,
            rightmap,
        })
    }

    pub fn resolve_equivalence(&self, eq: &Equivalence) -> Result<ResolvedEquivalence> {
        let Equivalence { left, right, .. } = eq;
        let left_err = Error::UndefinedGame(
            eq.left.to_string(),
            format!("in resolving the equivalence between {left} and {right}"),
        );
        let right_err = Error::UndefinedGame(
            eq.right.to_string(),
            format!("in resolving the equivalence between {left} and {right}"),
        );

        let left = self.get_game(&eq.left).ok_or(left_err)?.clone();
        let right = self.get_game(&eq.right).ok_or(right_err)?.clone();

        let inv_path = self.get_invariant_path(&eq.invariant_path);
        let invariant = std::fs::read_to_string(inv_path)?;

        let left_smt_file = self.get_smt_game_file(&eq.left)?;
        let right_smt_file = self.get_smt_game_file(&eq.right)?;
        let decl_smt_file = self.get_smt_decl_file(&eq.left, &eq.right)?;

        Ok(ResolvedEquivalence {
            left,
            right,
            invariant,
            left_smt_file,
            right_smt_file,
            decl_smt_file,
        })
    }

    pub fn resolve_assumption(&self, ass: &Assumption) -> Result<ResolvedAssumption> {
        let Assumption { left, right, .. } = ass;
        let left_err = Error::UndefinedGame(
            left.to_string(),
            format!("in resolving the assumption of {left} and {right}"),
        );
        let right_err = Error::UndefinedGame(
            right.to_string(),
            format!("in resolving the assumption of {left} and {right}"),
        );

        let left = self.get_game(left).ok_or(left_err)?.clone();
        let right = self.get_game(right).ok_or(right_err)?.clone();

        Ok(ResolvedAssumption { left, right })
    }
}
