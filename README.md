# Templative

## Installation

```sh
cargo install --path .
```

## Usage

This is a WIP. This is in the PoC stage and likely to change massively. Below is the current functionality.

It uses handlebars for templating.

Any files that end with `.tmpl` will be handled.

### Chunks:

if the tmpl extension is followed by an underscore and some text, then that is treated as a chunk id. Chunks are templated and inserted into the existing files based on their name.<br>
Example: file.txt.tmpl_some_thing has chunk id `some_thing`. The contents of this file will be inserted into file.txt in the output directory.<br>
The contents will be inserted at the chunk marker.

#### Chunk Markers:

A chunk marker is any line that contains the chunk id prefixed by `tmpl:`. Using the same example as above, the contents of the template file will be inserted into the target file at the line that contains `tmpl:some_thing`.
