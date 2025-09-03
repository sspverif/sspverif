#!/bin/bash

function fail() {
    echo $1 >&2
    touch failed
}

SSBEE=$(realpath $SSBEE)

echo "# Checking Reproducers still prove..."
for project_path in example-projects/repro-*; do
    project=$(basename $project_path)
    (
        echo "## Checking $project proves..."
        cd $project_path
        $SSBEE prove || fail "expected success, but failed: $project_path"
    )
done

echo "# Checking Error Reproducers still fail..."
for project_path in example-projects/err-repro-*; do
    project=$(basename $project_path)
    (
        echo "## Checking $project fails..."
        cd $project_path
        $SSBEE prove && fail "expected error, but succeeded: $project_path"
    )
done

echo "# Checking known-good proofs still prove..."
while read project; do
    project_path=example-projects/$project
    # skip comments
    if [ ! -d $project_path ]; then
        echo "WARN: skipping non-existing project in known-good proofs: $project"
        continue
    fi
    (
        echo "## Checking $project proves..."
        cd $project_path
        $SSBEE prove || fail "expected success, but failed: $project_path"
    )
done <example-projects/known-good.txt

if [ -f failed ]; then
    exit 1
fi
