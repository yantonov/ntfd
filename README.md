### Notification daemon

It is a tiny server which allow to set configurable notification.  

Idea: abstract client from the notification handling.  

#### Usage
Application expose HTTP API at default port 4242.  
You can trigger a notification:
```
    curl 'http://127.0.0.1/notify/key'
```
The key is used to find notification handler inside the configuration directory: (conf/key/run)  
run - executable script with any logic that you like  
See [examples](https://github.com/yantonov/ntfd/tree/master/examples/) configurations.  

#### Inspired by
1. [Paukan](https://youtu.be/n1Fsz-I8Qag?t=285)
2. [Napalm Death - You Suffer](https://www.youtube.com/watch?v=ybGOT4d2Hs8)
