[![Build Actions Status](https://github.com/yantonov/ntfd/workflows/ci/badge.svg)](https://github.com/yantonov/ntfd/actions)

### Notification daemon

It is a tiny HTTP server that provides interface to trigger notifications.  
In other words: it is a launcher with minimalistic HTTP interface.

#### Idea
To abstract a client from the notification handling details   
and to provide only the mechanism to trigger notifications.  

By the way, it's something similar to [webhook](https://github.com/adnanh/webhook)

#### Usage

Application exposes HTTP API on default port 4242.  
You can trigger a notification, for example, like that:
```
    curl -X POST 'http://127.0.0.1:4242/notify/key'
```
The key is used to find a notification handler inside the configuration directory:  
conf/key/run - an executable script which can contain any logic that you want.  
If there is no handler then the default handler will be used (conf/default/run).  
The key may contain letters, digits and underscores only.

The configuration directory is resolved next to the ntfd binary, not next to
the current working directory. The handler is executed directly, so it needs
the executable bit and a shebang line.

Directory structure:
```
    directory
        ntfd
        conf/
            handler1/
                    run
            handler2/
                    run
            default/
                    run
```

#### Endpoints

| Endpoint | Meaning |
| --- | --- |
| `POST /notify/<key>` | run the handler for `<key>` |
| `GET /health` | liveness check, answers with the pid |
| `GET /handlers` | list the configured handler keys |

The response to a notification carries the exit code and the captured output:
```json
    {"status": "Ok", "code": 0, "stdout": "...", "stderr": "..."}
```
The HTTP status follows the handler:

| Status | Meaning |
| --- | --- |
| 200 | the handler exited with 0 |
| 400 | the handler exited with a non-zero code, or the key is malformed |
| 413 | the request body is larger than 64k |
| 500 | no handler was found, the body is not valid JSON, or the handler could not be started |

A handler killed by a signal is reported as code 128 + signal, the way a
shell does it.

#### Payload

A JSON body is passed to the handler through the environment:
`NTFD_JSON_BODY` holds the raw body, and every first level field is exported
as `NTFD_JSON_FIELD_<NAME>`. See the [JSON payload](https://github.com/yantonov/ntfd/tree/master/examples/json_payload)
example for the details.

#### Options

```
    -p, --port <PORT>  port number (default = 4242)
    -b, --bind <BIND>  address to listen on (default = 127.0.0.1)
```

ntfd runs whatever its handlers do and has no authentication, so it listens
on loopback by default. Binding it to a reachable address hands anyone who
can connect the ability to run those handlers.

#### Inspired by
1. [Paukan](https://youtu.be/n1Fsz-I8Qag?t=285)
2. [Napalm Death - You Suffer](https://www.youtube.com/watch?v=ybGOT4d2Hs8)
3. [AnyBar](https://github.com/tonsky/AnyBar)

#### Examples
1. [You suffer](https://github.com/yantonov/ntfd/tree/master/examples/you_suffer).
2. [JSON payload](https://github.com/yantonov/ntfd/tree/master/examples/json_payload).
