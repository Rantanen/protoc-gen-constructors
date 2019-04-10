`protoc-gen-constructors`
=========================
### Generating constructors for Protobuf messages

## Building

```
$ cargo build
```

## Running

```
OUTLANG=java && protoc \
    --plugin=target/debug/protoc-gen-${OUTLANG}_constructors \
    --${OUTLANG}_out=out_dir \
    --${OUTLANG}_constructors_out=input.spec:out_dir \
    input.proto
```

The `LANG_out` and `LANG_constructors_out`paths must be equal. The
`LANG_constructors_out` also needs the constructor specification as a parameter.

## Constructor specification format

```
package foo.bar

message MyMessage {

    // Creates a text message.
    CreateMessage(
        // The text value to use for the message.
        string msg
    ) {
        text = msg
    }
    
    // Creates a text message with category.
    CreateMessageWithCategory(
        // The message category ID.
        int category,
        
        // The text value to use for the message.
        string msg
    ) {
        category = category
        text = msg
    }
}
```
