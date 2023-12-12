# Erased Buffers

![ci](https://github.com/s22s/erased-cells/actions/workflows/CI.yml/badge.svg)

Enables the use and manipulation of type-erased buffers of Rust primitives.

Please refer to the [documentation](https://s22s.github.io/erased-cells/erased_cells/) for details.


## Documentation

To generate docs:

    make docs

Output will be found in `docs/`.

To publish:

    make docs-publish

This will commit to and push from the `gh-pages` branch. After a few moments the results will be visible at the link above.

Note: If there's an error the first time you do this, you can try running `make docs-repair`. 
See [recipe](Makefile) for details.