# BINC

Rust implementation of the [binc file format](https://github.com/kurasu/binc).

This is a work in progress. Its a bit wilder than the Java version, as I'm trying out different things.
It includes various a operations that are not part of the version 1 specification, and a toy network protocol of
transferring operations. I will probably separate out a clean version of the version 1 specification soon.

Apart from a library itself, it includes a simple command-line tool which can also acts as a toy server, which is used
to try out network collaboration concepts.

There are also couple of gui apps (using egui) to try out things including binc explorer which is supposed to become
a generic viewer editor-thing.
