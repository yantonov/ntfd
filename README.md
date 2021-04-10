[![Build Actions Status](https://github.com/yantonov/ntfd/workflows/ci/badge.svg)](https://github.com/yantonov/ntfd/actions)

### Notification daemon

It is a tiny server which allow to configure notifications (notification handlers, to be more precise)..  

#### Idea
To abstract client from the notification handling details and to provide only the mechanism to trigger notifications.  

#### Usage
Application expose HTTP API at default port 4242.  
You can trigger a notification:
```
    curl 'http://127.0.0.1/notify/key'
```
The key is used to find a notification handler inside the configuration directory:  
(conf/key/run - executable script which can contain any logic that you want).  
If there is no handler then the default handler will be used (conf/default/run).  

Check [examples](https://github.com/yantonov/ntfd/tree/master/examples/) direcory.

#### Inspired by
1. [Paukan](https://youtu.be/n1Fsz-I8Qag?t=285)
2. [Napalm Death - You Suffer](https://www.youtube.com/watch?v=ybGOT4d2Hs8)
3. [AnyBar](https://github.com/tonsky/AnyBar)

#### TODO:
1. Accept JSON payload and pass fields as an environment variables to the handling script
