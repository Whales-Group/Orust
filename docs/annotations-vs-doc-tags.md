# Annotations and doc tags

ORust has two independent `@` syntaxes:

- A code annotation, such as `@data class User {}`, is parsed by the ORust
  grammar and can affect generated Rust.
- A doc tag, such as `/// @param id the database identifier`, is Markdown
  metadata inside a documentation comment. It never changes program behavior.

The same text outside a documentation comment is ordinary comment text. A
doc-tag parser must therefore run only on `OuterDocLine`, `OuterDocBlock`,
`InnerDocLine`, or `InnerDocBlock` comments. Unknown tags remain available as
text so documentation tooling does not silently discard author content.

The one intentional behavior-changing documentation mapping is a future
`@deprecated` tag, which may become a Rust `#[deprecated]` attribute on an
exported item. That mapping must be recorded in the generated metadata and
diagnosed when its syntax is invalid.
