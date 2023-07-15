# Seph


## Example systemd config

In e.g. `/etc/systemd/system/seph.service`

```
[Unit]
Description=Seph Daemon

[Service]
ExecStart=/usr/bin/seph-daemon

[Install]
WantedBy=multi-user.target
```
