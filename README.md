### Notification daemon

It is a tiny server which allow to set configurable notification.  

#### Idea
Abstract client from the notification handling and provider only the mechanism to trigger notifications.  

#### Usage
Application expose HTTP API at default port 4242.  
You can trigger a notification:
```
    curl 'http://127.0.0.1/notify/key'
```
The key is used to find notification handler inside the configuration directory: (conf/key/run).  
If there is no handler, default handler will be used (conf/default/run).  
run - executable script with any logic that you like.  
See [examples](https://github.com/yantonov/ntfd/tree/master/examples/) configurations.  

#### Inspired by
1. [Paukan](https://youtu.be/n1Fsz-I8Qag?t=285)
2. [Napalm Death - You Suffer](https://www.youtube.com/watch?v=ybGOT4d2Hs8)

#### TODO:
1. Accept JSON payload and pass fields as an environment variables to the handling script
