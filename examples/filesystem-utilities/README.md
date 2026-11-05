# Filesystem utilities

Run from the repository root:

```sh
tsrust run examples/filesystem-utilities/main.tsr
```

The example exclusively creates a temporary directory beneath the working
directory. It opens a read/write file, writes and truncates text, observes file
metadata, copies and renames UTF-8 text, then reads owned directory entries.
It removes its own temporary tree on the successful path. The output is:

```text
5
he
7
3
```

`createTempDir` transfers cleanup responsibility to the caller. Production code
should arrange cleanup on its failure paths too. Directory ordering and filesystem
errors depend on the host; this example observes only the entry count.
