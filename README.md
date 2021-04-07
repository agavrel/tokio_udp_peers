# Asynchronous Large file transfer over UDP, server/client in Rust

## About [User Data Protocol](https://en.wikipedia.org/wiki/User_Datagram_Protocol)

> UDP is suitable for purposes where error checking and correction are either not necessary or are performed in the application; UDP avoids the overhead of such processing in the protocol stack. Time-sensitive applications often use UDP because dropping packets is preferable to waiting for packets delayed due to retransmission, which may not be an option in a real-time system.[1]

#### UDP max data length

UDP packet is limited to 64kB (65536), however we need to take into account that each UDP's packet  
also has a UDP header of 8 bytes as well as an IP header of 20 bytes. Hence `MAX_DATA_LENGTH` is limited to 65,507 bytes.

```rs
const UDP_HEADER: u16 = 8;
const IP_HEADER: u16 = 20;
const MAX_DATA_LENGTH: u16 = 64 * 1024 - UDP_HEADER - IP_HEADER;
```

We have to remove bytes from our custom header which decrease our `MAX_CHUNK_SIZE` further by a few bytes (storing total number, index and filename):
```rs
const MAX_CHUNK_SIZE = MAX_DATA_LENGTH - AG_HEADER
```

**The following Program can send file up to about 4Gb** *(65535 (u16) chunks multiplied by chunk_size and divided by 1024^3 to convert from bytes to Gb).*  
This can be extended to much higher limits simply by using extra bytes in the custom header `AG_HEADER`.  

It also provides mechanisms to resend chunks that have failed to be received.

---

### Some words about [MTU](https://serverfault.com/questions/246508/how-is-the-mtu-is-65535-in-udp-but-ethernet-does-not-allow-frame-size-more-than) (Maximum Transfer Unit)

> UDP datagrams have little to do with the MTU size you can make them as big as you like up to the 64K is maximum mentioned above. You can even send one of them in an entire packet as long as you are using jumbo frames with a size larger the large datagram.

> However jumbo frames have to be supported by all the equipment the frame will pass over and this a problem. For practical purposes Ethernet frames are the most common tranport size, the MTU for these is circa 1500 bytes, I will say 1500 going forward, but it isn't always. When you create a UDP datagram larger than the underlying MTU (which as indicated is most often be ethernet) then it will be quietly be broken up into a number of 1500 byte frames. If you tcpdump this traffic you will see a number of packets broken at MTU boundary which will have the more fragments flag set along with a fragment number. The first packet will have a fragment number of 0 and the more fragments set and the last one will have a non-zero fragment number and more fragments not set.

> So why care? The implementation detail actually matters. Fragmentation can hurt performance in the network not a big issue anymore but one to be aware of. If a huge datagram size it used then should any fragment be lost the whole datagrams will need to be resent. Equally at high volumes and today these are perfectly achievable volumes then mis-association of frames at reassembly is possible. There can also be problems getting fragmented UDP packets to traverse enterprise firewall configurations where load balancers spread the packets out, if one fragment is on one firewall and the other on a different one then the traffic will get dropped as incomplete.

> So don't create UDP datagrams bigger than the MTU size fragmentation unless you have to and if you have to specify that the infrastructure being communicated between is close (same subnet close) at which point jumbo frames would likely be a good option.

---

## Setting up server over internet

#### Get public address

```
dig +short myip.opendns.com @resolver1.opendns.com
```

---

#### Get private address

```
hostname -I | awk -F' ' '{print $1}'
```

---

#### Setup router to forward port `Port` on our server private address

Go to your router homepage for settings, you will need to enter username and password
```
http://192.168.0.1/
```

---

## TODO List
* <s>Add filename information and detect true fileType based on file magic number</s> 03/18/21 done
* <s>Ability to encrypt data being sent</s> 03/24/21 done
* <s>Make it asynchronous using Tokio</s> 03/29/21 done
* <s>Debounce the function requesting the missing indexes</s> 04/07/21 done
* Make it peer-to-peer (there is no client/server file, every client is a potential server and vis-versa)
* build tracker server
* Flutter Frontend for the program
* ability to visualize blocks being received
* Extra: Compression of file before being sent *(not sure if worth if for jpg, only 5% saved, 2% for m4a, well that may still be interesting for movies...)*
* Send h264 with Cisco open source codec

---

## Miscellaneous

### Add crates to path
```
export PATH="/home/st4ck/.cargo/bin:$PATH"
```

### Check differing bytes

```
colordiff -y <(xxd 3.m4a) <(xxd 2.m4a)
```

### Send bytes to local port with command line

```
echo -n -e '\x00\x01\x00\x45\x01\x00\x00'  >/dev/udp/localhost/8080
```