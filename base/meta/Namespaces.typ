#link("/meta/users", "Users") can belong to many namespaces, and within each
namespace they can have a different #link("/meta/roles", "Roles").

== Meta namespace

Every user has access to the meta namespace. It contains the documentation for
the wiki, and is probably read only.

== Page URLs

```
/namespace/page-name         : Read "Page Name" in "Namespace"
/namespace/page-name/edit    : Edit "Page Name" in "Namespace"
/namespace/page-name/history : History of "Page Name" in "Namespace"
```
