This is an example of using the parameterized command.

You can write any logic based on the payload that is passed to the handler.

How to get payload:
1. Raw JSON body is passed through the environment variable NTFD_JSON_BODY.
2. All fields at the first level will be passed through environment variables
   with the prefix NTFD_JSON_FIELD_, for example NTFD_JSON_FIELD_FOO.

String values are passed as is, any other value keeps its JSON form:
a number arrives as `1`, a boolean as `true`, an object as `{"inner":1}`.

A field whose name is not made of letters, digits and underscores is left
out of the environment, because it cannot form a valid variable name. Such
fields are still available through NTFD_JSON_BODY.

The handler writes environment variables to the log.txt file.

The expected content of the log.txt file:
```
NTFD_JSON_BODY={"foo": "bar", "bar": "baz"}
NTFD_JSON_FIELD_FOO=bar
NTFD_JSON_FIELD_BAR=baz
```
