This is an example of using the parameterized command.

You can write any logic based on the payload that is passed to the handler.

How to get payload:
1. Raw JSON body is passed through the environment variable NTFD_JSON_RAW
2. All fields at the first level will be passed through environment variables with the prefix NTFD_JSON_FIELD_FOO.

The handler writes environment variables to the log.txt file.

The expected content of the log.txt file:
```
NTFD_JSON_BODY={"foo": "bar", "bar": "baz"}
NTFD_JSON_FIELD_FOO=bar
NTFD_JSON_FIELD_BAR=baz
```
