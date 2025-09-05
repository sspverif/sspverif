// SPDX-License-Identifier: MIT OR Apache-2.0

use std::{collections::HashSet, convert::Infallible};

use crate::{proof::GameInstance, types::Type};

use super::{
    resolveoracles, returnify, samplify, tableinitialize, treeify, type_extract, unwrapify,
    GameTransform, Transformation,
};

pub struct EquivalenceTransform;

impl super::ProofTransform for EquivalenceTransform {
    type Err = Infallible;

    type Aux = Vec<(
        String,
        (
            HashSet<Type>,
            samplify::SampleInfo,
            //split_partial::SplitInfo,
        ),
    )>;

    fn transform_proof<'a>(
        &self,
        proof: &'a crate::proof::Proof<'a>,
    ) -> Result<(crate::proof::Proof<'a>, Self::Aux), Self::Err> {
        let results = proof.instances().iter().map(transform_game_inst);
        let (instances, auxs) = itertools::process_results(results, |res| res.unzip())?;
        let proof = proof.with_new_instances(instances);

        Ok((proof, auxs))
    }
}

fn transform_game_inst(
    game_inst: &GameInstance,
) -> Result<
    (
        GameInstance,
        (
            String,
            (
                HashSet<Type>,
                samplify::SampleInfo,
                // split_partial::SplitInfo,
            ),
        ),
    ),
    Infallible,
> {
    let comp = game_inst.game();

    //let (comp, _) = resolvetypes::Transformation(comp)
    //    .transform()
    //    .expect("resolving user-defined types failed");
    let (comp, types) = type_extract::Transformation(comp)
        .transform()
        .expect("type extraction transformation failed unexpectedly");
    let (comp, samplinginfo) = samplify::Transformation(&comp)
        .transform()
        .expect("samplify transformation failed unexpectedly");
    let (comp, _) = unwrapify::Transformation(&comp)
        .transform()
        .expect("unwrapify transformation failed unexpectedly");
    let (comp, _) = resolveoracles::Transformation(&comp)
        .transform()
        .expect("resolveoracles transformation failed unexpectedly");
    let (comp, _) = returnify::TransformNg
        .transform_game(&comp)
        .expect("returnify transformation failed unexpectedly");
    let (comp, _) = treeify::Transformation(&comp)
        .transform()
        .expect("treeify transformation failed unexpectedly");
    // let (comp, splits) = split_partial::SplitPartial
    //     .transform_game(&comp)
    //     .expect("split_partial transform failed unexpectedly");
    // println!("#####################");
    // println!("{:#?}", comp);
    // println!("$$$$$$$$$$$$$$$$$$$$$");
    let (comp, _) = tableinitialize::Transformation(&comp)
        .transform()
        .expect("tableinitialize transformation failed unexpectedly");

    Ok((
        game_inst.with_other_game(comp),
        (game_inst.name().to_string(), (types, samplinginfo)),
    ))
}
