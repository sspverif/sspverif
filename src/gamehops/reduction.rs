use miette::Diagnostic;
use thiserror::Error;

use crate::parser::{
    reduction::{
        ReductionMapping as ParserReductionMapping,
        ReductionMappingEntry as ParserReductionMappingEntry
    },
    ast::Identifier,
};

/*
approach:

1. find the diff of left/right games and assumptions, recoding the path of signatures
2. for both left and right:
2.1. in the game, walk the path given by these signatures
2.2. check that the subgame starting by that root is identical


22-09-21 conceptualization
1. find diffs
2. check that diffs are the same package and params
    -> actually it might make sense to make this a separate function
3. in the game hop games, walk back the paths (diff->roots) from the assumption
4. use these as roots to a new composition (both left and right) and generate that (take care of exports)
5. compare to assumption

- a lot of this is comparing parts of the composition. Maybe we should add that as a function on the composition.
- what makes these comparisons tricky is that they don't need to be equal, they just need to be at least as strict as the assumption. It's okay if it offers less oracles to the adversary.
  -> this only concerns the exports


impl Composition {
    fn
}
 */

#[derive(Debug, Clone)]
pub(crate) struct Assumption {
    pub name: String,
    pub left_name: String,
    pub right_name: String,
}

#[derive(Clone, Debug)]
pub(crate) struct ReductionMappingEntry {
    assumption: String,
    construction: String,
}

impl ReductionMappingEntry {
    pub(crate) fn new(from: &ParserReductionMappingEntry) -> Self {
        Self {
            assumption: from.assumption().as_str().to_string(),
            construction: from.construction().as_str().to_string(),
        }
    }

    pub(crate) fn assumption(&self) -> &str {
        &self.assumption
    }

    pub(crate) fn construction(&self) -> &str {
        &self.construction
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ReductionMapping {
    assumption: String,
    construction: String,

    entries: Vec<ReductionMappingEntry>,
}

impl ReductionMapping {
    pub(crate) fn new(from: ParserReductionMapping) -> Self {
        Self {
            assumption: from.assumption_game_instance_name().as_str().to_string(),
            construction: from.construction_game_instance_name().as_str().to_string(),
            entries: from.entries().iter().map(|entry| {ReductionMappingEntry::new(entry)}).collect(),
        }
    }
    
    pub(crate) fn assumption_game_instance_name(&self) -> &str {
        &self.assumption
    }

    pub(crate) fn construction_game_instance_name(&self) -> &str {
        &self.construction
    }

    pub(crate) fn entries(&self) -> &[ReductionMappingEntry] {
        &self.entries
    }
}


#[derive(Debug, Clone)]
pub(crate) struct Reduction {
    left: ReductionMapping,
    right: ReductionMapping,

    assumption_name: String,
}

impl Reduction {
    pub(crate) fn new(
        left: ParserReductionMapping,
        right: ParserReductionMapping,
        assumption_name: String,
    ) -> Self {
        Self {
            left: ReductionMapping::new(left),
            right: ReductionMapping::new(right),
            assumption_name,
        }
    }

    pub(crate) fn left(&self) -> &ReductionMapping {
        &self.left
    }

    pub(crate) fn right(&self) -> &ReductionMapping {
        &self.right
    }

    pub(crate) fn assumption_name(&self) -> &str {
        &self.assumption_name
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum ReductionError {}
