== Database keys

=== History

The `namespace/slug/VERSION` record contains the version that should be
assigned to the next revision once an edit is made. It could also be thought
of as the live version of the page.

```
namespace/slug/1       => Diff 1 -> 2
namespace/slug/0       => Diff 0 -> 1
namespace/slug/VERSION => 2
```

=== Pages

```
namespace/slug         => Doc
```

=== Users

```
username               => User
```

=== Namespaces

```
namespace              => Namespace
```
