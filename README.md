Application that automatically adds to the routing table TCP connections that are blocked by the VPN.

***Note:*** Try more advanced techniques (like _net_cls_) before using this hacky solution.

## Usage

To use this application, you should first launch an instance with elevated privileges by running the following command:
```sh
sudo escape-vpn service
```
Or by creating a systemd service like this:
```
[Unit]
Description=Escape-VPN service

[Service]
ExecStart=/usr/bin/escape-vpn service

[Install]
WantedBy=multi-user.target
```

After the service is running, you can now use the command to attach or launch processes to be able to be VPN-escaped.
