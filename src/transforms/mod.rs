use crate::{
    package::{Composition, Package, PackageInstance},
    proof::{GameInstance, Proof},
    types::Type,
};

pub mod resolveoracles;
pub mod resolvetypes;
pub mod returnify;
pub mod samplify;
pub mod split_partial;
pub mod tableinitialize;
pub mod treeify;
pub mod type_extract;
pub mod unwrapify;

pub mod proof_transforms;

pub trait PackageTransform {
    type Err;
    type Aux;

    fn transform_package(&self, pkg: &Package) -> Result<(Package, Self::Aux), Self::Err>;
}

pub trait PackageInstanceTransform {
    type Err;
    type Aux;

    fn transform_package_instance(
        &self,
        inst: &PackageInstance,
    ) -> Result<(PackageInstance, Self::Aux), Self::Err>;
}

pub trait TypeTransform {
    type Err;
    type Aux;

    fn transform_type(&self, tipe: &Type) -> Result<(Type, Self::Aux), Self::Err>;
}
pub trait GameInstanceTransform {
    type Err;
    type Aux;

    fn transform_game_instance(
        &self,
        instance: &GameInstance,
    ) -> Result<(GameInstance, Self::Aux), Self::Err>;
}

pub trait GameTransform {
    type Err;
    type Aux;

    fn transform_game(&self, game: &Composition) -> Result<(Composition, Self::Aux), Self::Err>;
}

pub trait ProofTransform {
    type Err;
    type Aux;

    fn transform_proof(&self, proof: &Proof) -> Result<(Proof, Self::Aux), Self::Err>;
}

pub trait Transformation {
    type Err;
    type Aux;

    fn transform(&self) -> Result<(Composition, Self::Aux), Self::Err>;
}
/*
pub fn transform_all(
    comp: &Composition,
) -> Result<
    (
        Composition,
        <typecheck::Transformation as Transformation>::Aux,
        <type_extract::Transformation as Transformation>::Aux,
        <samplify::Transformation as Transformation>::Aux,
    ),
    <typecheck::Transformation as Transformation>::Err,
> {
    let (comp, _) = resolvetypes::Transformation(&comp)
        .transform()
        .expect("resolving user-defined types failed");
    let (comp, scope) = typecheck::Transformation::new_with_empty_scope(&comp).transform()?;
    let (comp, types) = type_extract::Transformation(&comp)
        .transform()
        .expect("type extraction transformation failed unexpectedly");
    let (comp, samplinginfo) = samplify::Transformation(&comp)
        .transform()
        .expect("samplify transformation failed unexpectedly");
    let (comp, _) = unwrapify::Transformation(&comp)
        .transform()
        .expect("unwrapify transformation failed unexpectedly");
    let (comp, _) = returnify::TransformNg
        .transform_game(&comp)
        .expect("returnify transformation failed unexpectedly");
    let (comp, _) = varspecify::Transformation(&comp)
        .transform()
        .expect("varspecify transformation failed unexpectedly");
    let (comp, _) = resolveoracles::Transformation(&comp)
        .transform()
        .expect("resolveoracles transformation failed unexpectedly");
    let (comp, _) = treeify::Transformation(&comp)
        .transform()
        .expect("treeify transformation failed unexpectedly");
    let (comp, _) = tableinitialize::Transformation(&comp)
        .transform()
        .expect("tableinitialize transformation failed unexpectedly");

    Ok((comp, scope, types, samplinginfo))
}

pub fn transform_explain(
    comp: &Composition,
) -> Result<
    (
        Composition,
        <typecheck::Transformation as Transformation>::Aux,
        <samplify::Transformation as Transformation>::Aux,
    ),
    <typecheck::Transformation as Transformation>::Err,
> {
    let (comp, _) = resolvetypes::Transformation(&comp)
        .transform()
        .expect("resolving user-defined types failed");
    let (comp, scope) = typecheck::Transformation::new_with_empty_scope(&comp).transform()?;
    let (comp, samplinginfo) = samplify::Transformation(&comp)
        .transform()
        .expect("samplify transformation failed unexpectedly");
    let (comp, _) = varspecify::Transformation(&comp)
        .transform()
        .expect("varspecify transformation failed unexpectedly");
    let (comp, _) = resolveoracles::Transformation(&comp)
        .transform()
        .expect("resolveoracles transformation failed unexpectedly");

    Ok((comp, scope, samplinginfo))
}
*/
