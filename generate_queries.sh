#!/bin/bash

graphql="./src/storage/github/queries/graphql"
schema="$graphql/schema.graphql"
rm -rf "$graphql/gen"
mkdir "$graphql/gen"
for file in $(find $graphql -type f -name 'query_*.graphql')
do
	graphql-client generate -o "$graphql/gen" --schema-path $schema $file
done
