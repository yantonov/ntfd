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
    curl 'http://127.0.0.1/notify/key'
```
The key is used to find a notification handler inside the configuration directory:  
conf/key/run - an executable script which can contain any logic that you want.  
If there is no handler then the default handler will be used (conf/default/run).  

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
#### Inspired by
1. [Paukan](https://youtu.be/n1Fsz-I8Qag?t=285)
2. [Napalm Death - You Suffer](https://www.youtube.com/watch?v=ybGOT4d2Hs8)
3. [AnyBar](https://github.com/tonsky/AnyBar)

#### Examples
1. [You suffer](https://github.com/yantonov/ntfd/tree/master/examples/you_suffer).
2. [JSON payload](https://github.com/yantonov/ntfd/tree/master/examples/json_payload).
