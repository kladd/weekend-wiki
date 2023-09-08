# weekend-wiki

Weekend project, don't use for real things: Token signing key is hard coded and there's no csrf mitigation.

Wiki documentation is in the wiki's [meta/ namespace](https://github.com/kladd/weekend-wiki/tree/main/base/meta).

## Prerequisites 
 - clang

## Done
1. CRU for pages
2. Ranked search for pages
3. Search index updates on edit/create
1. Writing and rendering markdown.
2. View page history.
3. namespaces
4. authentication and authorization

## TODO
1. token signing key not hardcoded
2. csrf mitigation

## Stack
- axum: web framework
- rocksdb: persistence
- tantivy: search/indexing
- askama: templates
- comrak: markdown
